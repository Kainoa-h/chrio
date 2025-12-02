Release steps:

- generate sqlx query files: cd into `./src-tauri` and run `cargo sqlx prepare`
- edit `./src/bindings.ts` and append "// @ts-nocheck" at the top
- run `bun run bump-version x.x.x` to bump version number
