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

---

### Development Workflow (using uv)

The project uses `uv` for package management. Always use uv to call tools and scripts!
When experiencing python packages issue always report back to user, never try to fix it yourself unless instructed!
Unless the requested change is trivial consult the implementation plan with the user.

# CODING CONVENTIONS

* Make sure to write documentation for each method/function. Make sure to update the documentation if the function signature changes.
* Only simple, one-time, obvious helper functions can be prefixed with _ and left without documentation.

* Write short, optimized code rather than lengthy verbose spaghetti.
* Generate functional and vectorized code whenever possible avoiding loops.
* Do not create nested loops in your code unless absolutely necessary!
* When creating CLI tools, use typer
* When dealing with paths prefer pathlib over os.path

* Use python 3.12+ typing hints whenever possible.
* Avoid duplicated code!

* Do not use magic numbers or strings. Define them as constants first.

# Documentation
Output docstrings in the following format:
```
def function_name(arg1: arg1_type, arg2: arg2_type) -> return_type:
    """
    Description of the function.

    :param arg1: Description of the first argument.
    :param arg2: Description of the second argument.
    :return: Description of the return value.
    """
    ...
```

### Testing & Code Quality

Run all linters and formatters in parallel only on modified files!
```bash
uv run ruff check --fix $files ; uv run black $files; uv run mypy $files; uv run basedpyright $files$

```
