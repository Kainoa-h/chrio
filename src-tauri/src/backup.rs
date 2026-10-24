//! Portable, verified archives and a restart-time restore journal.
use anyhow::{bail, ensure, Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tempfile::TempDir;
use tokio::sync::{Mutex, OwnedRwLockWriteGuard, RwLock, RwLockReadGuard};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const DATABASE: &str = "chrio.db";
const RESTORE: &str = ".chrio-restore";
const DB_FILES: [&str; 4] = [DATABASE, "chrio.db-wal", "chrio.db-shm", "chrio.db-journal"];
const IMAGE_COLUMNS: [&str; 4] = ["anterior", "posterior", "right_lateral", "left_lateral"];
const MAX_MANIFEST: u64 = 16 * 1024 * 1024;

// Filesystem replacement is safe only when no other Chrio process has an open pool.
pub struct DataDirectoryLock {
    _file: File,
}
pub async fn lock_data_directory(root: &Path) -> Result<DataDirectoryLock> {
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(".chrio-data.lock"))?;
    // A restart spawns the replacement process just before the old process exits.
    for attempt in 0..=20 {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(DataDirectoryLock { _file: file }),
            Err(error)
                if error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
                    && attempt < 20 =>
            {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(error) => {
                return Err(error).context(
                    "Chrio data is already in use. Close the other Chrio process and try again",
                )
            }
        }
    }
    unreachable!()
}

#[derive(Default)]
pub struct BackupState {
    gate: Arc<RwLock<()>>,
    session: Mutex<Option<BackupSession>>,
    startup_message: Option<String>,
}
struct BackupSession {
    _guard: OwnedRwLockWriteGuard<()>,
    staged: Option<TempDir>,
}
impl BackupState {
    pub fn with_message(startup_message: Option<String>) -> Self {
        Self {
            startup_message,
            ..Self::default()
        }
    }
    pub fn access(&self) -> std::result::Result<RwLockReadGuard<'_, ()>, String> {
        self.gate
            .try_read()
            .map_err(|_| "Import / Export is in progress. Try again when it finishes.".into())
    }
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct DataSummary {
    pub clients: u32,
    pub sessions: u32,
    pub photos: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u32,
    created_at: String,
    app_version: String,
    files: BTreeMap<String, FileRecord>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {
    size: u64,
    sha256: String,
}

fn portable_path(name: &str) -> Result<PathBuf> {
    ensure!(
        !name.is_empty() && !name.contains(['\\', ':', '\0']),
        "Unsafe archive path: {name}"
    );
    ensure!(
        name.split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".."),
        "Unsafe archive path: {name}"
    );
    Ok(name.split('/').collect())
}
fn archive_name(path: &Path) -> Result<String> {
    let name = path
        .components()
        .map(|part| {
            part.as_os_str()
                .to_str()
                .context("Photo filename is not valid Unicode")
        })
        .collect::<Result<Vec<_>>>()?
        .join("/");
    portable_path(&name)?;
    Ok(name)
}
fn list_photos(root: &Path) -> Result<Vec<String>> {
    fn walk(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<()> {
        ensure!(
            !fs::symlink_metadata(dir)?.file_type().is_symlink(),
            "Photo directories cannot be symbolic links"
        );
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                walk(root, &entry.path(), files)?;
            } else {
                ensure!(
                    kind.is_file(),
                    "Photos must be regular files: {}",
                    entry.path().display()
                );
                files.push(format!(
                    "photos/{}",
                    archive_name(entry.path().strip_prefix(root)?)?
                ));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    if root.exists() {
        walk(root, root, &mut files)?;
    }
    files.sort();
    Ok(files)
}
fn digest_file(path: &Path) -> Result<FileRecord> {
    let mut input = File::open(path)?;
    let mut digest = Sha256::new();
    let mut size = 0;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
        size += n as u64;
    }
    Ok(FileRecord {
        size,
        sha256: format!("{:x}", digest.finalize()),
    })
}
fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    Ok(())
}
fn sync_file(path: &Path) -> Result<()> {
    // FlushFileBuffers on Windows requires a writable handle.
    File::options()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(())
}
fn sync_tree(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            sync_tree(&entry.path())?;
        } else {
            sync_file(&entry.path())?;
        }
    }
    sync_dir(root)
}
async fn open_database(path: &Path) -> Result<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .foreign_keys(true),
        )
        .await?)
}
async fn check_database(pool: &SqlitePool) -> Result<()> {
    let integrity: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_all(pool)
        .await?;
    ensure!(integrity == ["ok"], "Database integrity check failed");
    ensure!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(pool)
            .await?
            .is_empty(),
        "Database contains broken client/session relationships"
    );
    let orphans: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions LEFT JOIN clients ON clients.id = sessions.client_id WHERE clients.id IS NULL").fetch_one(pool).await?;
    ensure!(orphans == 0, "Database contains sessions without a client");
    let objects: Vec<(String, String)> =
        sqlx::query_as("SELECT type, name FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'")
            .fetch_all(pool)
            .await?;
    for (kind, name) in objects {
        ensure!(
            kind == "table" && ["clients", "sessions", "_sqlx_migrations"].contains(&name.as_str()),
            "Unsupported database object: {name}"
        );
    }
    // Read all required columns; do not accept a merely SQLite-shaped archive.
    crate::models::Client::find_all(pool).await?;
    sqlx::query_as::<_, crate::models::Session>("SELECT id, client_id, datetime, session_number, height, weight, anterior, posterior, right_lateral, left_lateral, notes, anterior_crop, posterior_crop, right_lateral_crop, left_lateral_crop FROM sessions")
        .fetch_all(pool).await?;
    Ok(())
}
async fn summary(pool: &SqlitePool, photos: usize) -> Result<DataSummary> {
    Ok(DataSummary {
        clients: sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM clients")
            .fetch_one(pool)
            .await?
            .try_into()?,
        sessions: sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
            .fetch_one(pool)
            .await?
            .try_into()?,
        photos: photos.try_into().context("Too many photos")?,
    })
}
async fn rewrite_images(
    pool: &SqlitePool,
    files: &HashSet<String>,
    source_root: Option<&Path>,
    destination_root: Option<&Path>,
) -> Result<()> {
    let rows =
        sqlx::query("SELECT id, anterior, posterior, right_lateral, left_lateral FROM sessions")
            .fetch_all(pool)
            .await?;
    let mut tx = pool.begin().await?;
    for row in rows {
        let id: i64 = row.try_get("id")?;
        for column in IMAGE_COLUMNS {
            if let Some(value) = row.try_get::<Option<String>, _>(column)? {
                let relative = if let Some(root) = source_root {
                    let path = Path::new(&value);
                    ensure!(
                        path.is_absolute(),
                        "Photo reference is not an absolute local path: {value}"
                    );
                    format!(
                        "photos/{}",
                        archive_name(
                            path.strip_prefix(root.join("photos"))
                                .context("Referenced photo is outside Chrio's photos folder")?
                        )?
                    )
                } else {
                    value
                };
                portable_path(&relative)?;
                ensure!(
                    relative.starts_with("photos/") && files.contains(&relative),
                    "Referenced photo is missing: {relative}"
                );
                let replacement = match destination_root {
                    Some(root) => root
                        .join(portable_path(&relative)?)
                        .to_str()
                        .context("App data path is not valid Unicode")?
                        .to_owned(),
                    None => relative,
                };
                sqlx::query(&format!("UPDATE sessions SET {column} = ? WHERE id = ?"))
                    .bind(replacement)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }
    tx.commit().await?;
    Ok(())
}

async fn export_archive(pool: &SqlitePool, root: &Path, destination: &Path) -> Result<()> {
    // A snapshot includes committed WAL contents; copying the live .db alone does not.
    let work = tempfile::Builder::new()
        .prefix(".chrio-backup-")
        .tempdir_in(root)?;
    let snapshot = work.path().join(DATABASE);
    sqlx::query("VACUUM main INTO ?")
        .bind(snapshot.to_str().context("Invalid snapshot path")?)
        .execute(pool)
        .await?;
    let photos = list_photos(&root.join("photos"))?;
    let snapshot_pool = open_database(&snapshot).await?;
    let checked = async {
        check_database(&snapshot_pool).await?;
        rewrite_images(
            &snapshot_pool,
            &photos.iter().cloned().collect(),
            Some(root),
            None,
        )
        .await
    }
    .await;
    snapshot_pool.close().await;
    checked?;
    let root = root.to_owned();
    let destination = destination.to_owned();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut manifest = Manifest {
            format: "chrio-client-data".into(),
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            app_version: env!("CARGO_PKG_VERSION").into(),
            files: BTreeMap::new(),
        };
        manifest
            .files
            .insert(DATABASE.into(), digest_file(&snapshot)?);
        for name in &photos {
            manifest
                .files
                .insert(name.clone(), digest_file(&root.join(portable_path(name)?))?);
        }
        let parent = destination
            .parent()
            .context("Export destination has no parent directory")?;
        let mut output = tempfile::NamedTempFile::new_in(parent)?;
        {
            let mut writer = ZipWriter::new(output.as_file_mut());
            for (name, record) in &manifest.files {
                writer.start_file(
                    name,
                    SimpleFileOptions::default()
                        .compression_method(CompressionMethod::Stored)
                        .large_file(record.size >= u32::MAX as u64),
                )?;
                let path = if name == DATABASE {
                    snapshot.clone()
                } else {
                    root.join(portable_path(name)?)
                };
                std::io::copy(&mut File::open(path)?, &mut writer)?;
            }
            writer.start_file(
                "manifest.json",
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )?;
            writer.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
            writer.finish()?;
        }
        output.as_file().sync_all()?;
        verify_archive(output.path(), None)?;
        output.persist(&destination).map_err(|e| e.error)?;
        sync_dir(parent)?;
        drop(work);
        Ok(())
    })
    .await??;
    Ok(())
}

// Verification streams every entry. Extraction happens only into a new private directory.
fn verify_archive(path: &Path, extract: Option<&Path>) -> Result<Manifest> {
    let input = File::open(path)?;
    let mut directory = input.try_clone()?;
    let mut archive = ZipArchive::new(input).context("Not a readable ZIP archive")?;
    // zip 2 indexes entries by name and silently hides exact duplicates. Count the raw
    // central-directory records as well, so an ambiguous archive cannot pass validation.
    directory.seek(SeekFrom::Start(archive.central_directory_start()))?;
    let mut records = 0;
    loop {
        let mut header = [0u8; 46];
        directory.read_exact(&mut header[..4])?;
        if &header[..4] != b"PK\x01\x02" {
            break;
        }
        directory.read_exact(&mut header[4..])?;
        records += 1;
        ensure!(records <= archive.len(), "Duplicate archive entries");
        let variable_size: u64 = [28, 30, 32]
            .iter()
            .map(|&offset| u16::from_le_bytes([header[offset], header[offset + 1]]) as u64)
            .sum();
        directory.seek(SeekFrom::Current(variable_size as i64))?;
    }
    ensure!(
        records == archive.len(),
        "Invalid archive central directory"
    );
    let manifest: Manifest = {
        let entry = archive
            .by_name("manifest.json")
            .context("This is not a Chrio backup: manifest.json is missing")?;
        ensure!(
            entry.size() <= MAX_MANIFEST,
            "Archive manifest is too large"
        );
        serde_json::from_reader(entry.take(MAX_MANIFEST + 1)).context("Invalid backup manifest")?
    };
    ensure!(
        manifest.format == "chrio-client-data" && manifest.version == 1,
        "Unsupported Chrio archive format/version"
    );
    ensure!(
        manifest.files.contains_key(DATABASE),
        "Backup database is missing"
    );
    ensure!(
        archive.len() == manifest.files.len() + 1,
        "Archive has missing or unexpected entries"
    );
    let mut seen = HashSet::new();
    // Case-folded collisions are rejected for portability to case-insensitive filesystems.
    let mut folded = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let relative = portable_path(&name)?;
        ensure!(
            seen.insert(name.clone()) && folded.insert(name.to_lowercase()),
            "Duplicate archive path: {name}"
        );
        ensure!(
            !entry.is_dir()
                && entry
                    .unix_mode()
                    .map(|mode| mode & 0o170000 == 0o100000 || mode & 0o170000 == 0)
                    .unwrap_or(true),
            "Archive contains a non-regular file"
        );
        ensure!(
            entry.compression() == CompressionMethod::Stored,
            "Unsupported archive compression"
        );
        if name == "manifest.json" {
            continue;
        }
        ensure!(
            name == DATABASE || name.starts_with("photos/"),
            "Unexpected archive path: {name}"
        );
        let record = manifest
            .files
            .get(&name)
            .context("Archive entry is not listed in the manifest")?;
        ensure!(entry.size() == record.size, "Incorrect file size: {name}");
        let mut output = if let Some(root) = extract {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().context("Invalid archive parent")?)?;
            Some(File::options().write(true).create_new(true).open(path)?)
        } else {
            None
        };
        let mut digest = Sha256::new();
        let mut size = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = entry.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            size = size
                .checked_add(n as u64)
                .context("Archive file is too large")?;
            ensure!(
                size <= record.size,
                "Archive file exceeds its declared size"
            );
            digest.update(&buffer[..n]);
            if let Some(file) = output.as_mut() {
                file.write_all(&buffer[..n])?;
            }
        }
        ensure!(
            size == record.size && format!("{:x}", digest.finalize()) == record.sha256,
            "Checksum mismatch: {name}"
        );
        if let Some(file) = output {
            file.sync_all()?;
        }
    }
    ensure!(
        manifest.files.keys().all(|key| seen.contains(key)),
        "Archive is incomplete"
    );
    if let Some(root) = extract {
        fs::create_dir_all(root.join("photos"))?;
        sync_tree(root)?;
    }
    Ok(manifest)
}

async fn stage_archive(archive: PathBuf, root: &Path) -> Result<(TempDir, DataSummary)> {
    let stage = tempfile::Builder::new()
        .prefix(".chrio-backup-")
        .tempdir_in(root)?;
    let (stage, mut manifest) =
        tokio::task::spawn_blocking(move || -> Result<(TempDir, Manifest)> {
            let manifest = verify_archive(&archive, Some(stage.path()))?;
            Ok((stage, manifest))
        })
        .await??;
    let pool = open_database(&stage.path().join(DATABASE)).await?;
    let result = async {
        // Check for unexpected objects before running migrations against untrusted input.
        let forbidden: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' AND (type != 'table' OR name NOT IN ('clients', 'sessions', '_sqlx_migrations'))").fetch_one(&pool).await?;
        ensure!(forbidden == 0, "Archive database has unsupported objects");
        sqlx::migrate!("./migrations").run(&pool).await.context("Unsupported or invalid backup database schema")?;
        check_database(&pool).await?;
        let photos = list_photos(&stage.path().join("photos"))?;
        rewrite_images(&pool, &photos.iter().cloned().collect(), None, Some(root)).await?;
        summary(&pool, photos.len()).await
    }.await;
    pool.close().await;
    let counts = result?;
    // Paths and migrations change the staged database; record its final bytes for startup verification.
    manifest
        .files
        .insert(DATABASE.into(), digest_file(&stage.path().join(DATABASE))?);
    fs::write(
        stage.path().join("manifest.json"),
        serde_json::to_vec(&manifest)?,
    )?;
    sync_tree(stage.path())?;
    Ok((stage, counts))
}

fn verify_dataset_files(root: &Path, manifest_path: &Path) -> Result<()> {
    let manifest: Manifest = serde_json::from_reader(File::open(manifest_path)?)?;
    for (name, expected) in manifest.files {
        let actual = digest_file(&root.join(portable_path(&name)?))?;
        ensure!(
            actual.size == expected.size && actual.sha256 == expected.sha256,
            "Staged or restored file failed verification: {name}"
        );
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    // New recovery copies use writable app-owned permissions even for read-only originals.
    let mut output = File::create(destination)?;
    std::io::copy(&mut File::open(source)?, &mut output)?;
    output.sync_all()?;
    Ok(())
}
#[cfg(windows)]
fn clear_readonly(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            clear_readonly(&entry?.path())?;
        }
    }
    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(source)?.file_type().is_symlink(),
        "Cannot back up a symbolic link directory"
    );
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        } else {
            ensure!(kind.is_file(), "Cannot back up non-regular data file");
            copy_file(&entry.path(), &destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn remove_dataset(root: &Path) -> Result<()> {
    for name in DB_FILES {
        let path = root.join(name);
        if path.exists() {
            #[cfg(windows)]
            clear_readonly(&path)?;
            fs::remove_file(path)?;
        }
    }
    let photos = root.join("photos");
    if photos.exists() {
        #[cfg(windows)]
        clear_readonly(&photos)?;
        fs::remove_dir_all(photos)?;
    }
    sync_dir(root)
}
fn copy_dataset(source: &Path, destination: &Path) -> Result<()> {
    ensure!(
        fs::symlink_metadata(source.join(DATABASE))?
            .file_type()
            .is_file(),
        "Current database is missing or is not a regular file"
    );
    fs::create_dir_all(destination)?;
    for name in DB_FILES {
        let path = source.join(name);
        if path.exists() {
            copy_file(&path, &destination.join(name))?;
        }
    }
    if source.join("photos").exists() {
        copy_tree(&source.join("photos"), &destination.join("photos"))?;
    }
    sync_tree(destination)
}
fn write_phase(restore: &Path, phase: &str) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(restore)?;
    file.write_all(phase.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(restore.join("phase")).map_err(|e| e.error)?;
    sync_dir(restore)
}
fn rollback(root: &Path, restore: &Path) -> Result<()> {
    // Keep the previous copy intact until rollback is durable, so rollback itself is restartable.
    ensure!(
        restore.join("previous").join(DATABASE).is_file(),
        "Recovery database is missing; restore files were retained"
    );
    remove_dataset(root)?;
    copy_dataset(&restore.join("previous"), root)?;
    write_phase(restore, "rolled-back")
}
async fn verify_installed(root: &Path) -> Result<()> {
    let pool = open_database(&root.join(DATABASE)).await?;
    let result = async {
        sqlx::migrate!("./migrations").run(&pool).await?;
        check_database(&pool).await?;
        let photos = list_photos(&root.join("photos"))?;
        rewrite_images(
            &pool,
            &photos.iter().cloned().collect(),
            Some(root),
            Some(root),
        )
        .await
    }
    .await;
    pool.close().await;
    result
}

/// Called before opening the normal database pool. No live database files are touched until
/// the previous dataset has been fully copied and synced and the applying phase is durable.
pub async fn recover_restore(root: &Path) -> Result<Option<String>> {
    recover_restore_with(root, |_| Ok(())).await
}
async fn recover_restore_with(
    root: &Path,
    checkpoint: impl Fn(&str) -> Result<()>,
) -> Result<Option<String>> {
    // Cleanup directories are never part of recovery; deleting one may itself be interrupted.
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if (entry
            .file_name()
            .to_string_lossy()
            .starts_with(".chrio-restore-cleanup-")
            || entry
                .file_name()
                .to_string_lossy()
                .starts_with(".chrio-backup-"))
            && entry.file_type()?.is_dir()
        {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
    let restore = root.join(RESTORE);
    if !restore.exists() {
        return Ok(None);
    }
    let phase = fs::read_to_string(restore.join("phase"))
        .context("Cannot read restore journal; recovery files were retained")?;
    let message = match phase.as_str() {
        "ready" => {
            let result = async {
                verify_dataset_files(&restore, &restore.join("manifest.json"))?;
                copy_dataset(root, &restore.join("previous"))?;
                checkpoint("previous-copied")?;
                write_phase(&restore, "applying")?;
                checkpoint("applying")?;
                remove_dataset(root)?;
                checkpoint("live-removed")?;
                fs::rename(restore.join(DATABASE), root.join(DATABASE))?;
                checkpoint("database-installed")?;
                fs::rename(restore.join("photos"), root.join("photos"))?;
                checkpoint("photos-installed")?;
                sync_dir(root)?;
                verify_dataset_files(root, &restore.join("manifest.json"))?;
                verify_installed(root).await?;
                sync_tree(&root.join("photos"))?;
                sync_file(&root.join(DATABASE))?;
                checkpoint("verified")?;
                write_phase(&restore, "committed")?;
                checkpoint("committed")?;
                Ok::<_, anyhow::Error>(())
            }
            .await;
            match result {
                Ok(()) => "Client data imported successfully.".to_owned(),
                Err(error) => {
                    // A failure before 'applying' leaves the live dataset untouched.
                    let phase = fs::read_to_string(restore.join("phase"))?;
                    if phase == "committed" {
                        format!("Client data imported successfully. Cleanup notice: {error:#}")
                    } else {
                        if phase == "applying" {
                            rollback(root, &restore)?;
                        } else {
                            // A failed preparation must never be retried on a later startup
                            // after the user has resumed adding data to the original dataset.
                            write_phase(&restore, "rolled-back")?;
                        }
                        format!("Import failed; your previous data was kept: {error:#}")
                    }
                }
            }
        }
        "applying" => {
            rollback(root, &restore)?;
            "An interrupted import was recovered. Your previous data has been restored.".to_owned()
        }
        "committed" => "Client data imported successfully.".to_owned(),
        "rolled-back" => {
            "Your previous data has been restored after an interrupted import.".to_owned()
        }
        _ => bail!("Unrecognized restore journal; recovery files were retained"),
    };
    // Rename the whole journal before deleting anything. A crash during cleanup must never
    // leave a half-deleted journal that blocks the next startup.
    let cleanup = root.join(format!(".chrio-restore-cleanup-{}", std::process::id()));
    if let Err(error) = fs::rename(&restore, &cleanup) {
        return Ok(Some(format!(
            "{message} Recovery files could not be cleaned up: {error}"
        )));
    }
    if let Err(error) = sync_dir(root) {
        return Ok(Some(format!(
            "{message} Recovery files were retained because cleanup could not be synced: {error}"
        )));
    }
    checkpoint("cleanup-renamed")?;
    let _ = fs::remove_dir_all(cleanup);
    Ok(Some(message))
}

#[tauri::command]
#[specta::specta]
pub async fn begin_backup(
    app: AppHandle,
    db: State<'_, crate::db::DbState>,
    state: State<'_, BackupState>,
) -> std::result::Result<DataSummary, String> {
    let mut session = state.session.lock().await;
    ensure_session_empty(&session)?;
    let guard = state.gate.clone().write_owned().await;
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let photos = list_photos(&root.join("photos")).map_err(|e| e.to_string())?;
    let counts = summary(&db, photos.len())
        .await
        .map_err(|e| e.to_string())?;
    *session = Some(BackupSession {
        _guard: guard,
        staged: None,
    });
    Ok(counts)
}
fn ensure_session_empty(session: &Option<BackupSession>) -> std::result::Result<(), String> {
    if session.is_some() {
        Err("Import / Export is already open".into())
    } else {
        Ok(())
    }
}
#[tauri::command]
#[specta::specta]
pub async fn end_backup(state: State<'_, BackupState>) -> std::result::Result<(), String> {
    state.session.lock().await.take();
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn export_backup(
    app: AppHandle,
    db: State<'_, crate::db::DbState>,
    state: State<'_, BackupState>,
) -> std::result::Result<Option<String>, String> {
    let session = state.session.lock().await;
    if session.is_none() {
        return Err("Open Import / Export first".into());
    }
    let dialog_app = app.clone();
    let selected = tokio::task::spawn_blocking(move || {
        dialog_app
            .dialog()
            .file()
            .add_filter("Chrio ZIP backup", &["zip"])
            .set_file_name(format!(
                "chrio-backup-{}.zip",
                chrono::Local::now().format("%Y-%m-%d-%H%M%S")
            ))
            .blocking_save_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(path) = selected else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|e| e.to_string())?;
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    // Never publish an archive into the dataset it is backing up.
    let parent = path
        .parent()
        .ok_or("Invalid destination")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if parent.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
        return Err("Choose an export location outside Chrio's app data folder".into());
    }
    export_archive(&db, &root, &path)
        .await
        .map_err(|e| format!("Export failed: {e:#}"))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}
#[tauri::command]
#[specta::specta]
pub async fn validate_backup(
    app: AppHandle,
    state: State<'_, BackupState>,
) -> std::result::Result<Option<DataSummary>, String> {
    let mut session = state.session.lock().await;
    let session = session.as_mut().ok_or("Open Import / Export first")?;
    session.staged = None;
    let dialog_app = app.clone();
    let selected = tokio::task::spawn_blocking(move || {
        dialog_app
            .dialog()
            .file()
            .add_filter("Chrio ZIP backup", &["zip"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;
    let Some(path) = selected else {
        return Ok(None);
    };
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let (stage, counts) = stage_archive(path.into_path().map_err(|e| e.to_string())?, &root)
        .await
        .map_err(|e| format!("Cannot import this archive: {e:#}"))?;
    session.staged = Some(stage);
    Ok(Some(counts))
}
#[tauri::command]
#[specta::specta]
pub async fn import_backup(
    app: AppHandle,
    db: State<'_, crate::db::DbState>,
    state: State<'_, BackupState>,
) -> std::result::Result<(), String> {
    let mut session = state.session.lock().await;
    let session = session.as_mut().ok_or("Open Import / Export first")?;
    let stage = session
        .staged
        .as_ref()
        .ok_or("Select and validate an archive first")?;
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let restore = root.join(RESTORE);
    if restore.exists() {
        return Err("A previous restore needs recovery. Restart Chrio before importing.".into());
    }
    // Publish the complete, durable stage and its journal with one directory rename.
    sync_tree(stage.path()).map_err(|e| e.to_string())?;
    write_phase(stage.path(), "ready").map_err(|e| e.to_string())?;
    fs::rename(stage.path(), &restore).map_err(|e| e.to_string())?;
    // Once published, keep the backend locked and restart even if directory sync reports an error.
    let _ = sync_dir(&root);
    db.close().await;
    app.restart();
}
#[tauri::command]
#[specta::specta]
pub fn backup_startup_message(state: State<'_, BackupState>) -> Option<String> {
    state.startup_message.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fixture(root: &Path, name: &str, populated: bool) -> SqlitePool {
        fs::create_dir_all(root).unwrap();
        File::create(root.join(DATABASE)).unwrap();
        let pool = open_database(&root.join(DATABASE)).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .unwrap();
        if populated {
            sqlx::query("INSERT INTO clients (id, firstname, lastname, dob, sex, registration_date) VALUES (42, ?, '姓', '1980-02-29', 'F', '2025-01-01T12:34:56+08:00')")
                .bind(name).execute(&pool).await.unwrap();
            let mut paths = Vec::new();
            for (i, column) in IMAGE_COLUMNS.iter().enumerate() {
                let path = root
                    .join("photos")
                    .join("42_照片")
                    .join("session_7")
                    .join(format!("{column}.jpg"));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, [i as u8, 0, 255, 33, 42]).unwrap();
                paths.push(path.to_str().unwrap().to_owned());
            }
            sqlx::query("INSERT INTO sessions (id, client_id, datetime, session_number, height, weight, anterior, posterior, right_lateral, left_lateral, notes, anterior_crop, posterior_crop, right_lateral_crop, left_lateral_crop) VALUES (98, 42, '2025-02-03T04:05:06+08:00', 7, 172.5, 65.2, ?, ?, ?, ?, 'Notes\nwith Unicode 照片', '{\"x\":0.1}', '{\"x\":0.2}', '{\"x\":0.3}', '{\"x\":0.4}')")
                .bind(&paths[0]).bind(&paths[1]).bind(&paths[2]).bind(&paths[3]).execute(&pool).await.unwrap();
            fs::write(
                root.join("photos").join("unreferenced.jpg"),
                b"also preserve me",
            )
            .unwrap();
        }
        pool
    }
    fn publish(stage: &TempDir, root: &Path) {
        write_phase(stage.path(), "ready").unwrap();
        fs::rename(stage.path(), root.join(RESTORE)).unwrap();
        sync_dir(root).unwrap();
    }
    fn entries(path: &Path) -> Vec<(String, Vec<u8>)> {
        let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
        (0..archive.len())
            .map(|i| {
                let mut entry = archive.by_index(i).unwrap();
                let name = entry.name().to_owned();
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).unwrap();
                (name, bytes)
            })
            .collect()
    }
    fn write_zip(path: &Path, entries: &[(String, Vec<u8>)]) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        for (name, bytes) in entries {
            // Force ZIP64 headers on tiny files, exercising ZIP64 without large fixtures.
            zip.start_file(
                name,
                SimpleFileOptions::default()
                    .compression_method(CompressionMethod::Stored)
                    .large_file(true),
            )
            .unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    fn update_manifest(entries: &mut [(String, Vec<u8>)], change: impl FnOnce(&mut Manifest)) {
        let (_, bytes) = entries
            .iter_mut()
            .find(|(name, _)| name == "manifest.json")
            .unwrap();
        let mut manifest: Manifest = serde_json::from_slice(bytes).unwrap();
        change(&mut manifest);
        *bytes = serde_json::to_vec(&manifest).unwrap();
    }
    async fn archive_fixture() -> (TempDir, PathBuf) {
        let source = tempfile::tempdir().unwrap();
        let pool = fixture(source.path(), "Original", true).await;
        let archive = source.path().join("backup.zip");
        export_archive(&pool, source.path(), &archive)
            .await
            .unwrap();
        pool.close().await;
        (source, archive)
    }

    #[tokio::test]
    async fn round_trip_preserves_every_field_and_photo_and_excludes_settings() {
        let source = tempfile::tempdir().unwrap();
        let pool = fixture(source.path(), "First name", true).await;
        fs::write(
            source.path().join("camera-settings.json"),
            b"source settings",
        )
        .unwrap();
        let original_client =
            serde_json::to_value(crate::models::Client::find_all(&pool).await.unwrap()).unwrap();
        let original_session = crate::models::Session::find_by_id(&pool, 98)
            .await
            .unwrap()
            .unwrap();
        let archive = source.path().join("backup.zip");
        // The source pool stays open and uses WAL throughout export.
        export_archive(&pool, source.path(), &archive)
            .await
            .unwrap();
        let manifest = verify_archive(&archive, None).unwrap();
        assert_eq!(manifest.files.len(), 6);
        assert!(!manifest.files.keys().any(|name| name.contains("settings")));
        let destination = tempfile::tempdir().unwrap();
        let old = fixture(destination.path(), "Replace me", true).await;
        old.close().await;
        fs::write(
            destination.path().join("camera-settings.json"),
            b"destination settings",
        )
        .unwrap();
        let (stage, counts) = stage_archive(archive, destination.path()).await.unwrap();
        assert_eq!(
            counts,
            DataSummary {
                clients: 1,
                sessions: 1,
                photos: 5
            }
        );
        publish(&stage, destination.path());
        assert!(recover_restore(destination.path())
            .await
            .unwrap()
            .unwrap()
            .contains("successfully"));
        assert!(!destination.path().join(RESTORE).exists());
        assert_eq!(
            fs::read(destination.path().join("camera-settings.json")).unwrap(),
            b"destination settings"
        );
        let restored = open_database(&destination.path().join(DATABASE))
            .await
            .unwrap();
        assert_eq!(
            original_client,
            serde_json::to_value(crate::models::Client::find_all(&restored).await.unwrap())
                .unwrap()
        );
        let restored_session = crate::models::Session::find_by_id(&restored, 98)
            .await
            .unwrap()
            .unwrap();
        let mut expected = serde_json::to_value(original_session).unwrap();
        let actual = serde_json::to_value(restored_session).unwrap();
        for column in IMAGE_COLUMNS {
            let old_path = Path::new(expected[column].as_str().unwrap());
            let new_path = destination
                .path()
                .join(old_path.strip_prefix(source.path()).unwrap());
            assert_eq!(fs::read(old_path).unwrap(), fs::read(&new_path).unwrap());
            expected[column] = serde_json::Value::String(new_path.to_str().unwrap().into());
        }
        assert_eq!(expected, actual);
        assert_eq!(
            fs::read(destination.path().join("photos/unreferenced.jpg")).unwrap(),
            b"also preserve me"
        );
        let new_id = sqlx::query("INSERT INTO clients (firstname, lastname, dob, sex, registration_date) VALUES ('Next', 'Client', '2000-01-01', 'M', 'now')")
            .execute(&restored).await.unwrap().last_insert_rowid();
        assert!(new_id > 42);
        restored.close().await;
        pool.close().await;
    }

    #[tokio::test]
    async fn empty_dataset_and_zip64_round_trip() {
        let source = tempfile::tempdir().unwrap();
        let pool = fixture(source.path(), "", false).await;
        let archive = source.path().join("empty.zip");
        export_archive(&pool, source.path(), &archive)
            .await
            .unwrap();
        pool.close().await;
        let contents = entries(&archive);
        write_zip(&archive, &contents);
        let destination = tempfile::tempdir().unwrap();
        let (stage, counts) = stage_archive(archive, destination.path()).await.unwrap();
        assert_eq!(
            counts,
            DataSummary {
                clients: 0,
                sessions: 0,
                photos: 0
            }
        );
        assert!(stage.path().join("photos").is_dir());
    }

    #[tokio::test]
    async fn missing_source_photo_does_not_replace_existing_export() {
        let source = tempfile::tempdir().unwrap();
        let pool = fixture(source.path(), "Broken", true).await;
        fs::remove_file(source.path().join("photos/42_照片/session_7/anterior.jpg")).unwrap();
        let archive = source.path().join("existing.zip");
        fs::write(&archive, b"existing backup").unwrap();
        assert!(export_archive(&pool, source.path(), &archive)
            .await
            .is_err());
        assert_eq!(fs::read(archive).unwrap(), b"existing backup");
        pool.close().await;
    }

    #[tokio::test]
    async fn corrupted_truncated_missing_and_unsupported_archives_are_rejected() {
        let (source, archive) = archive_fixture().await;
        let original = entries(&archive);
        let bad = source.path().join("bad.zip");
        let mut modified = original.clone();
        modified
            .iter_mut()
            .find(|(name, _)| name.starts_with("photos/"))
            .unwrap()
            .1[0] ^= 1;
        write_zip(&bad, &modified);
        assert!(verify_archive(&bad, None).is_err());
        let bytes = fs::read(&archive).unwrap();
        fs::write(&bad, &bytes[..bytes.len() / 2]).unwrap();
        assert!(verify_archive(&bad, None).is_err());
        modified = original.clone();
        modified.retain(|(name, _)| !name.ends_with("anterior.jpg"));
        write_zip(&bad, &modified);
        assert!(verify_archive(&bad, None).is_err());
        modified = original.clone();
        update_manifest(&mut modified, |manifest| manifest.version = 999);
        write_zip(&bad, &modified);
        assert!(verify_archive(&bad, None).is_err());
    }

    #[tokio::test]
    async fn malicious_paths_collisions_and_symlinks_are_rejected() {
        let (source, archive) = archive_fixture().await;
        let original = entries(&archive);
        let bad = source.path().join("bad.zip");
        for name in [
            "../escape",
            "/absolute",
            "photos/../../escape",
            "photos/./alias",
            "photos\\\\escape",
            "photos/C:/escape",
        ] {
            let mut modified = original.clone();
            modified.push((name.into(), vec![]));
            update_manifest(&mut modified, |manifest| {
                manifest.files.insert(
                    name.into(),
                    FileRecord {
                        size: 0,
                        sha256: format!("{:x}", Sha256::digest([])),
                    },
                );
            });
            write_zip(&bad, &modified);
            assert!(verify_archive(&bad, None).is_err(), "Accepted {name}");
        }
        let mut modified = original.clone();
        modified.push(("photos/UNREFERENCED.JPG".into(), b"collision".to_vec()));
        update_manifest(&mut modified, |manifest| {
            manifest.files.insert(
                "photos/UNREFERENCED.JPG".into(),
                FileRecord {
                    size: 9,
                    sha256: format!("{:x}", Sha256::digest(b"collision")),
                },
            );
        });
        write_zip(&bad, &modified);
        assert!(verify_archive(&bad, None).is_err());
        let mut zip = ZipWriter::new(File::create(&bad).unwrap());
        for (name, bytes) in original {
            if name == "photos/unreferenced.jpg" {
                zip.add_symlink(name, "../../../escape", SimpleFileOptions::default())
                    .unwrap();
            } else {
                zip.start_file(
                    name,
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
                )
                .unwrap();
                zip.write_all(&bytes).unwrap();
            }
        }
        zip.finish().unwrap();
        assert!(verify_archive(&bad, None).is_err());
    }

    #[tokio::test]
    async fn exact_duplicate_zip_entries_are_rejected_even_when_library_hides_them() {
        let (source, archive) = archive_fixture().await;
        let mut modified = entries(&archive);
        modified.push((
            "photos/UNREFERENCED.JPG".into(),
            b"also preserve me".to_vec(),
        ));
        let bad = source.path().join("duplicates.zip");
        write_zip(&bad, &modified);
        let mut bytes = fs::read(&bad).unwrap();
        let alias = b"photos/UNREFERENCED.JPG";
        let original = b"photos/unreferenced.jpg";
        for start in 0..=bytes.len() - alias.len() {
            if &bytes[start..start + alias.len()] == alias {
                bytes[start..start + alias.len()].copy_from_slice(original);
            }
        }
        fs::write(&bad, bytes).unwrap();
        assert!(verify_archive(&bad, None)
            .unwrap_err()
            .to_string()
            .contains("Duplicate"));
    }

    #[tokio::test]
    async fn invalid_database_and_broken_references_are_rejected_before_restore() {
        let (source, archive) = archive_fixture().await;
        let mut modified = entries(&archive);
        modified
            .iter_mut()
            .find(|(name, _)| name == DATABASE)
            .unwrap()
            .1 = b"not a database".to_vec();
        update_manifest(&mut modified, |manifest| {
            manifest.files.insert(
                DATABASE.into(),
                FileRecord {
                    size: 14,
                    sha256: format!("{:x}", Sha256::digest(b"not a database")),
                },
            );
        });
        let bad = source.path().join("bad-db.zip");
        write_zip(&bad, &modified);
        let destination = tempfile::tempdir().unwrap();
        fs::write(destination.path().join("sentinel"), b"unchanged").unwrap();
        assert!(stage_archive(bad, destination.path()).await.is_err());
        assert_eq!(
            fs::read(destination.path().join("sentinel")).unwrap(),
            b"unchanged"
        );
        assert!(!destination.path().join(RESTORE).exists());
    }

    async fn changed_database_archive(
        source: &Path,
        archive: &Path,
        statements: &[&str],
    ) -> PathBuf {
        let mut modified = entries(archive);
        let database = source.join("changed.db");
        fs::write(
            &database,
            &modified
                .iter()
                .find(|(name, _)| name == DATABASE)
                .unwrap()
                .1,
        )
        .unwrap();
        let pool = open_database(&database).await.unwrap();
        for statement in statements {
            sqlx::query(statement).execute(&pool).await.unwrap();
        }
        pool.close().await;
        let bytes = fs::read(&database).unwrap();
        let record = digest_file(&database).unwrap();
        modified
            .iter_mut()
            .find(|(name, _)| name == DATABASE)
            .unwrap()
            .1 = bytes;
        update_manifest(&mut modified, |manifest| {
            manifest.files.insert(DATABASE.into(), record);
        });
        let changed = source.join("changed.zip");
        write_zip(&changed, &modified);
        changed
    }

    #[tokio::test]
    async fn older_schema_is_migrated_in_staging_and_newer_schema_is_rejected() {
        let (source, archive) = archive_fixture().await;
        let old = changed_database_archive(
            source.path(),
            &archive,
            &[
                "ALTER TABLE sessions DROP COLUMN anterior_crop",
                "ALTER TABLE sessions DROP COLUMN posterior_crop",
                "ALTER TABLE sessions DROP COLUMN right_lateral_crop",
                "ALTER TABLE sessions DROP COLUMN left_lateral_crop",
                "DELETE FROM _sqlx_migrations WHERE version = 20251201074407",
            ],
        )
        .await;
        let destination = tempfile::tempdir().unwrap();
        let (stage, counts) = stage_archive(old, destination.path()).await.unwrap();
        assert_eq!(counts.sessions, 1);
        let pool = open_database(&stage.path().join(DATABASE)).await.unwrap();
        let crop: Option<String> = sqlx::query_scalar("SELECT anterior_crop FROM sessions")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(crop, None);
        pool.close().await;
        let newer = changed_database_archive(source.path(), &archive, &[
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) SELECT 99999999999999, description, installed_on, success, checksum, execution_time FROM _sqlx_migrations LIMIT 1",
        ]).await;
        assert!(stage_archive(newer, destination.path()).await.is_err());
    }

    #[tokio::test]
    async fn missing_photo_references_broken_relationships_and_triggers_are_rejected() {
        let (source, archive) = archive_fixture().await;
        let destination = tempfile::tempdir().unwrap();
        for statements in [
            vec!["UPDATE sessions SET anterior = 'photos/missing.jpg'"],
            vec![
                "PRAGMA foreign_keys = OFF",
                "UPDATE sessions SET client_id = 999",
            ],
            vec![
                "CREATE TRIGGER unexpected AFTER INSERT ON clients BEGIN DELETE FROM clients; END",
            ],
        ] {
            let changed = changed_database_archive(source.path(), &archive, &statements).await;
            assert!(stage_archive(changed, destination.path()).await.is_err());
            assert!(!destination.path().join(RESTORE).exists());
        }
    }

    #[tokio::test]
    async fn every_interrupted_install_recovers_one_complete_dataset() {
        let (_source, archive) = archive_fixture().await;
        for point in [
            "previous-copied",
            "applying",
            "live-removed",
            "database-installed",
            "photos-installed",
            "verified",
            "committed",
            "cleanup-renamed",
        ] {
            let destination = tempfile::tempdir().unwrap();
            let pool = fixture(destination.path(), "Previous", true).await;
            pool.close().await;
            fs::write(
                destination.path().join("photos/unreferenced.jpg"),
                b"previous photo",
            )
            .unwrap();
            fs::write(destination.path().join("preferences"), b"keep settings").unwrap();
            let (stage, _) = stage_archive(archive.clone(), destination.path())
                .await
                .unwrap();
            publish(&stage, destination.path());
            let root = destination.path().to_owned();
            let task = tokio::spawn(async move {
                recover_restore_with(&root, |at| {
                    if at == point {
                        panic!("Simulated process interruption at {at}");
                    }
                    Ok(())
                })
                .await
            });
            assert!(task.await.unwrap_err().is_panic());
            recover_restore(destination.path()).await.unwrap();
            assert!(!destination.path().join(RESTORE).exists());
            let pool = open_database(&destination.path().join(DATABASE))
                .await
                .unwrap();
            check_database(&pool).await.unwrap();
            let name: String = sqlx::query_scalar("SELECT firstname FROM clients WHERE id = 42")
                .fetch_one(&pool)
                .await
                .unwrap();
            let new_survives = ["previous-copied", "committed", "cleanup-renamed"].contains(&point);
            assert_eq!(
                name,
                if new_survives { "Original" } else { "Previous" },
                "checkpoint: {point}"
            );
            assert_eq!(
                fs::read(destination.path().join("photos/unreferenced.jpg")).unwrap(),
                if new_survives {
                    b"also preserve me".as_slice()
                } else {
                    b"previous photo".as_slice()
                }
            );
            assert_eq!(
                fs::read(destination.path().join("preferences")).unwrap(),
                b"keep settings"
            );
            pool.close().await;
        }
    }

    #[tokio::test]
    async fn failed_install_rolls_back_without_waiting_for_another_restart() {
        let (_source, archive) = archive_fixture().await;
        for point in [
            "previous-copied",
            "applying",
            "live-removed",
            "database-installed",
            "photos-installed",
            "verified",
        ] {
            let root = tempfile::tempdir().unwrap();
            let pool = fixture(root.path(), "Previous", true).await;
            pool.close().await;
            let (stage, _) = stage_archive(archive.clone(), root.path()).await.unwrap();
            publish(&stage, root.path());
            let message = recover_restore_with(root.path(), |at| {
                ensure!(at != point, "Injected I/O failure (including disk full)");
                Ok(())
            })
            .await
            .unwrap()
            .unwrap();
            assert!(message.contains("previous data was kept"));
            let pool = open_database(&root.path().join(DATABASE)).await.unwrap();
            let name: String = sqlx::query_scalar("SELECT firstname FROM clients")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(name, "Previous");
            pool.close().await;
        }
    }

    #[tokio::test]
    async fn failed_preparation_is_never_reapplied_after_cleanup_failure() {
        let (_source, archive) = archive_fixture().await;
        let root = tempfile::tempdir().unwrap();
        let pool = fixture(root.path(), "Previous", true).await;
        pool.close().await;
        let (stage, _) = stage_archive(archive, root.path()).await.unwrap();
        publish(&stage, root.path());
        let blocker = root
            .path()
            .join(format!(".chrio-restore-cleanup-{}", std::process::id()));
        let message = recover_restore_with(root.path(), |at| {
            if at == "previous-copied" {
                fs::write(&blocker, b"prevents cleanup rename")?;
                bail!("Injected preparation failure");
            }
            Ok(())
        })
        .await
        .unwrap()
        .unwrap();
        assert!(message.contains("previous data was kept"));
        assert_eq!(
            fs::read_to_string(root.path().join(RESTORE).join("phase")).unwrap(),
            "rolled-back"
        );
        let pool = open_database(&root.path().join(DATABASE)).await.unwrap();
        sqlx::query("UPDATE clients SET firstname = 'Added after failure'")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        fs::remove_file(blocker).unwrap();
        recover_restore(root.path()).await.unwrap();
        let pool = open_database(&root.path().join(DATABASE)).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT firstname FROM clients")
                .fetch_one(&pool)
                .await
                .unwrap(),
            "Added after failure"
        );
        pool.close().await;
    }

    #[tokio::test]
    async fn interrupted_rollback_can_be_repeated_from_intact_previous_copy() {
        let root = tempfile::tempdir().unwrap();
        let pool = fixture(root.path(), "Previous", true).await;
        pool.close().await;
        let restore = root.path().join(RESTORE);
        fs::create_dir(&restore).unwrap();
        copy_dataset(root.path(), &restore.join("previous")).unwrap();
        write_phase(&restore, "applying").unwrap();
        remove_dataset(root.path()).unwrap();
        fs::write(root.path().join(DATABASE), b"partial rollback").unwrap();
        recover_restore(root.path()).await.unwrap();
        let pool = open_database(&root.path().join(DATABASE)).await.unwrap();
        check_database(&pool).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT firstname FROM clients")
                .fetch_one(&pool)
                .await
                .unwrap(),
            "Previous"
        );
        pool.close().await;
    }

    #[tokio::test]
    async fn backend_gate_rejects_access_until_backup_finishes() {
        let state = BackupState::default();
        assert!(state.access().is_ok());
        let guard = state.gate.clone().write_owned().await;
        assert!(state.access().is_err());
        drop(guard);
        assert!(state.access().is_ok());
    }

    #[tokio::test]
    async fn data_directory_lock_waits_for_the_previous_process_to_release() {
        let root = tempfile::tempdir().unwrap();
        let lock = lock_data_directory(root.path()).await.unwrap();
        let release = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            drop(lock);
        });
        let replacement = lock_data_directory(root.path()).await.unwrap();
        release.await.unwrap();
        drop(replacement);
    }

    #[tokio::test]
    async fn concurrent_app_processes_cannot_open_the_same_dataset() {
        let root = tempfile::tempdir().unwrap();
        let lock = lock_data_directory(root.path()).await.unwrap();
        let second = File::options()
            .read(true)
            .write(true)
            .open(root.path().join(".chrio-data.lock"))
            .unwrap();
        assert!(second.try_lock_exclusive().is_err());
        drop(lock);
        second.try_lock_exclusive().unwrap();
    }

    #[tokio::test]
    async fn staged_corruption_keeps_current_data_and_abandoned_stages_are_cleaned() {
        let (_source, archive) = archive_fixture().await;
        let root = tempfile::tempdir().unwrap();
        let pool = fixture(root.path(), "Previous", true).await;
        pool.close().await;
        let (stage, _) = stage_archive(archive, root.path()).await.unwrap();
        fs::write(stage.path().join("photos/unreferenced.jpg"), b"corrupted").unwrap();
        publish(&stage, root.path());
        let abandoned = root.path().join(".chrio-backup-abandoned");
        fs::create_dir(&abandoned).unwrap();
        fs::write(abandoned.join("partial"), b"temporary data").unwrap();
        let message = recover_restore(root.path()).await.unwrap().unwrap();
        assert!(message.contains("previous data was kept"));
        assert!(!abandoned.exists());
        let pool = open_database(&root.path().join(DATABASE)).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT firstname FROM clients")
                .fetch_one(&pool)
                .await
                .unwrap(),
            "Previous"
        );
        pool.close().await;
    }

    #[test]
    fn paths_are_portable_and_confined() {
        for bad in [
            "",
            "/root",
            "../root",
            "photos//a",
            "photos/./a",
            "photos/../a",
            "C:/root",
            "photos/a\\\\b",
            "photos/a\\0b",
        ] {
            assert!(portable_path(bad).is_err(), "Accepted {bad:?}");
        }
        assert_eq!(
            portable_path("photos/42_照片/image.jpg").unwrap(),
            PathBuf::from("photos/42_照片/image.jpg")
        );
    }
}
