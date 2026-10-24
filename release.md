Release steps:

- generate sqlx query files: cd into `./src-tauri` and run `cargo sqlx prepare`
- edit `./src/bindings.ts` and append "// @ts-nocheck" at the top
- run `bun run bump-version x.x.x` to bump version number

macOS releases use Tauri's ad-hoc signing identity (`-`). The workflow verifies
the app signatures, but users must still approve the app in Privacy & Security.
See the README for installation steps and a workaround for older downloads.

Camera access also requires `NSCameraUsageDescription` in the bundled Info.plist
and `com.apple.security.device.camera` in the signed app's entitlements. These
are configured in `src-tauri/Info.plist` and `src-tauri/Entitlements.plist`; the
release workflow checks both in the built app. Test camera access from the
rebuilt `.app` bundle, since signing changes do not update an installed copy.
If access was previously denied, enable Chrio in System Settings → Privacy &
Security → Camera, then quit and reopen the app.

For distribution without the Gatekeeper approval step, configure Developer ID
signing and notarization following https://v2.tauri.app/distribute/sign/macos/.
The workflow currently does not pass Apple certificate or notarization secrets.
