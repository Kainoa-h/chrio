# Chrio

*This application was largely vibecoded with Google Gemini 3 & 2.5*

A local-first, application for chiropractic and physical therapy image comparison and session tracking.

**Chrio** is designed to help practitioners securely document patient progress. It allows you to store client profiles, log sessions with notes and biometric data (height/weight), and importantly, capture and compare clinical images (anterior, posterior, lateral views) side-by-side with cropping tools.

## Features

* **Client Management:** Create and manage client profiles with basic demographic info.
* **Session Tracking:** Log detailed sessions including date, notes, height, and weight.
* **Image Capture & Processing:** Capture clinical images directly within the app or import them. Includes built-in cropping tools.
* **Visual Comparison:** Compare images from different sessions side-by-side with synchronized controls to visualize progress over time.
* **Local & Private:** All data is stored locally on your device using an embedded SQLite database & local file system. No cloud uploads, ensuring patient data privacy.

## Before 1.0.0 Release

* [ ] Test Cases!
* [ ] Refactor and clean up AI slop
* [ ] Logging and error handling
* [ ] Backup & restore functionality
* [ ] General UI/UX improvements
  * [ ] tool-tips on hover
  * [x] show age instead of bday?
  * [ ] client metrics: session counts, client counts, ...
  * [ ] navigational improvements (kb shortcuts, larger navigation buttons/areas, guestures)
  * [ ] persist user's camera choice
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

Ensure you have `Rust`, `Bun` (or Node.js), and typical build tools installed.

1. **Install dependencies:**

    ```bash
    bun install
    ```

2. **Run in development mode:**

    ```bash
    bun run tauri dev
    ```

## License

This project is dedicated to the public domain under the [0BSD License](https://opensource.org/license/0bsd).
You are free to use, copy, modify, and distribute this software for any purpose with or without fee.

---
*Provided "as is" without warranty of any kind. For clinical documentation purposes only.*
