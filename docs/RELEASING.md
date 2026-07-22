# Releasing plotine to crates.io

Target cadence: **one minor release every 4–6 weeks**, plus patch releases for
regressions.

## Prerequisites

1. crates.io account with publish rights for `plotine*` crates
2. `cargo login` (token in `~/.cargo/credentials.toml`)
3. Clean `main`, CI green on the commit you intend to ship
4. `CHANGELOG.md` updated; version bumped in workspace `Cargo.toml`

## Version bump

Edit root `Cargo.toml`:

```toml
[workspace.package]
version = "0.5.0"   # bump here; all members inherit
```

Workspace path deps already pin `version = "0.5.0"` — keep them in lockstep.

## Preflight

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p plotine --features polars,ndarray,evcxr
# Linux: cargo test -p plotine --test visual_snapshots
mdbook build book
.\scripts\publish.ps1 -DryRun
```

`cargo publish --dry-run` for `plotine-core` should succeed. Dependent crates will
fail dry-run until `plotine-core` (then render/text/backends) actually exist on
crates.io — that is expected for a first multi-crate publish. Publish in the
order below without long gaps.

Default branch note: keep `main` as the public entry. After hardening commits land
on `release/v0.4.x`, merge that branch into `main` before tagging / publishing so
anonymous clones are not stuck on the Initial commit.

## Publish order

Dependencies must land before dependents:

1. `plotine-core`
2. `plotine-render`
3. `plotine-text`
4. `plotine-backend-skia`
5. `plotine-backend-svg`
6. `plotine-backend-pdf`
7. `plotine-backend-pgf`
8. `plotine`
9. `plotine-pyplot`

```powershell
.\scripts\publish.ps1           # real publish
# or step through (always pass --registry crates-io if you use a mirror):
# cargo publish -p plotine-core --registry crates-io
# …
```

Wait ~60s between crates if crates.io index lag causes “not found” errors.

## After publish

1. Verify <https://crates.io/crates/plotine> and <https://docs.rs/plotine>
2. `git tag v0.5.0` && `git push origin v0.5.0` (retag only if the existing tag
   does not match the published commit)
3. GitHub Release with CHANGELOG section
4. Mark the version released in `CHANGELOG.md`

## Yanking

Only yank for security or “does not compile on supported targets”. Prefer a
patch release otherwise.
