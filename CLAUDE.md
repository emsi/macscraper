### Development Workflow (Tauri / Rust / Svelte)

**Rust backend:**
```bash
cargo test --manifest-path src-tauri/Cargo.toml   # run all Rust tests
cargo build --manifest-path src-tauri/Cargo.toml  # check compilation
```

**Frontend (TypeScript / Svelte):**
```bash
npm run test        # Vitest unit tests
npm run dev         # Vite dev server only (no Tauri window)
npm run tauri dev   # full Tauri dev mode (opens desktop window)
npm run tauri build # production build
```

