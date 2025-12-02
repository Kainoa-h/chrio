import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';
import { execSync } from 'child_process';

// Get the new version from command line arguments
const newVersion = process.argv[2];

if (!newVersion) {
  console.error('Please provide a version number: bun bump-version.ts <version>');
  process.exit(1);
}

// Validate version format (simple regex for x.y.z)
if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error('Invalid version format. Please use x.y.z (e.g., 1.0.0)');
  process.exit(1);
}

const paths = {
  packageJson: join(process.cwd(), 'package.json'),
  tauriConf: join(process.cwd(), 'src-tauri', 'tauri.conf.json'),
  cargoToml: join(process.cwd(), 'src-tauri', 'Cargo.toml'),
};

// Update package.json
try {
  const packageJson = JSON.parse(readFileSync(paths.packageJson, 'utf-8'));
  packageJson.version = newVersion;
  writeFileSync(paths.packageJson, JSON.stringify(packageJson, null, 2) + '\n');
  console.log(`Updated package.json to ${newVersion}`);
} catch (error) {
  console.error('Error updating package.json:', error);
}

// Update tauri.conf.json
try {
  const tauriConf = JSON.parse(readFileSync(paths.tauriConf, 'utf-8'));
  tauriConf.version = newVersion;
  writeFileSync(paths.tauriConf, JSON.stringify(tauriConf, null, 2) + '\n');
  console.log(`Updated src-tauri/tauri.conf.json to ${newVersion}`);
} catch (error) {
  console.error('Error updating tauri.conf.json:', error);
}

// Update Cargo.toml using cargo command if available, or regex fallback
try {
    // Try using cargo-edit if installed (more robust), otherwise manual replace
    // Using regex replacement for Cargo.toml to preserve comments and structure
    let cargoToml = readFileSync(paths.cargoToml, 'utf-8');
    // Replace version = "x.y.z" in the [package] section
    // We look for the first occurrence which is typically the package version
    cargoToml = cargoToml.replace(/^version = ".*"$/m, `version = "${newVersion}"`);
    writeFileSync(paths.cargoToml, cargoToml);
    console.log(`Updated src-tauri/Cargo.toml to ${newVersion}`);
} catch (error) {
  console.error('Error updating Cargo.toml:', error);
}

console.log(`\nSuccessfully bumped version to ${newVersion}`);
