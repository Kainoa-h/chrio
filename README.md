# Chrio

*This application was largely vibecoded with Google Gemini 3 & 2.5*

A local-first, application for chiropractic and physical therapy image comparison and session tracking.

**Chrio** is designed to help practitioners securely document patient progress. It allows you to store client profiles, log sessions with notes and biometric data (height/weight), and importantly, capture and compare clinical images (anterior, posterior, lateral views) side-by-side with cropping tools.

## Features

* **Client Management:** Create and manage client profiles with basic demographic info.
* **Session Tracking:** Log detailed sessions including date, notes, height, and weight.
* **Image Capture & Processing:** Capture clinical images directly within the app or import them. Includes built-in cropping tools.
* **Camera Settings:** Choose a camera, adjust its saved 2:3 framing, and request streaming and capture resolutions. Preferences are remembered per camera. Capture saves the full video frame and stores its framing separately. Higher-resolution capture preserves the preview's aspect ratio, then restores the preview resolution. If the requested resolution is unavailable, a validated full preview frame is offered instead. The crop editor shows the original image, and framing edits update only crop metadata.
* **Visual Comparison:** Compare images from different sessions side-by-side with synchronized controls to visualize progress over time.
* **Import / Export:** Use the archive button beside Settings to export all client records, sessions, notes, measurements, crop information, and original photos into a verified ZIP backup. Camera and app settings are excluded. Import validates the complete archive, offers to export existing data first, and requires confirmation before replacing all client data and restarting. Interrupted restores recover the previous dataset. Backups are unencrypted and can be large; keep sufficient free disk space for staging and the recovery copy.
* **Local & Private:** All data is stored locally on your device using an embedded SQLite database & local file system. No cloud uploads, ensuring patient data privacy.

## Before 1.0.0 Release

* [ ] Test Cases!
* [ ] Refactor and clean up AI slop
* [ ] Logging and error handling
* [x] Backup & restore functionality
* [ ] General UI/UX improvements
  * [ ] tool-tips on hover
  * [x] show age instead of bday?
  * [ ] client metrics: session counts, client counts, ...
  * [ ] navigational improvements (kb shortcuts, larger navigation buttons/areas, guestures)
  * [x] persist user's camera choice
  * [ ] editable crop mask image/svg
  * [ ] have an actual app icon/logo
  * [ ] dynamic weighted ordering of clients
* [ ] report generation?

## Tech Stack

* **Frontend:** [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Tailwind CSS](https://tailwindcss.com/) + [Shadcn Vue](https://www.shadcn-vue.com/) (components)

* **Backend:** [Rust](https://www.rust-lang.org/) (via [Tauri](https://tauri.app/))

* **Database:** [SQLite](https://www.sqlite.org/index.html) (via [SQLx](https://github.com/launchbadge/sqlx))

* **Icons:** [Lucide](https://lucide.dev/)

## macOS installation

Download the `aarch64` release for Apple Silicon or `x86_64` for Intel, extract
the app archive, and move `chrio.app` to Applications.

Releases use ad-hoc code signing and are not notarized by Apple. On first launch,
macOS may block the app; open **System Settings > Privacy & Security** and choose
**Open Anyway** after attempting to launch it.

If an older release reports that the app is damaged, and you trust the download,
remove its download quarantine attribute and try opening it again:

```bash
xattr -dr com.apple.quarantine /Applications/chrio.app
```

This applies only to Chrio; it does not disable Gatekeeper globally.

## Development Setup

Open **Settings** beside About, or the settings button inside the camera dialog, to configure a camera. Drag the frame or its corner to adjust it, then choose **Save settings**. Arrow keys move the frame; Alt + arrows resize it, and Shift makes larger adjustments. Imported images continue to use the per-photo crop editor. Enable **Mirror image previews** to mirror live video, saved and imported photos, comparisons, and the framing editor. This device-wide preference is off by default and changes only the display; captures and saved originals stay unchanged.

Ensure you have `Rust`, `Bun` (or Node.js), and typical build tools installed.

1. **Install dependencies:**

    ```bash
    bun install
    ```

2. **Run in development mode:**

    ```bash
    bun run tauri dev
    ```

Run frontend tests with `bun test`, Rust archive/recovery tests with `cd src-tauri && cargo test --lib`, and type-check/build with `bun run build` (or run these commands through `nix develop --command`).

To regenerate typed frontend commands without launching the app, run `cd src-tauri && cargo run --example export_bindings`.

## License

This project is dedicated to the public domain under the [0BSD License](https://opensource.org/license/0bsd).
You are free to use, copy, modify, and distribute this software for any purpose with or without fee.

---
*Provided "as is" without warranty of any kind. For clinical documentation purposes only.*
