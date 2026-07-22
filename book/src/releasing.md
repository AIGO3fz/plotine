# Releasing

Cadence target: **one crates.io minor every 4–6 weeks** (or sooner for critical fixes).

Full maintainer checklist: repository `docs/RELEASING.md`.

## Quick path

```powershell
# from repo root (Windows)
.\scripts\publish.ps1 -DryRun          # cargo publish --dry-run for all crates
.\scripts\publish.ps1                  # real publish (needs crates.io token)
```

Publish order (dependencies first):

1. `plotine-core`
2. `plotine-render`
3. `plotine-text`
4. `plotine-backend-skia`
5. `plotine-backend-svg`
6. `plotine-backend-pdf`
7. `plotine-backend-pgf`
8. `plotine`
9. `plotine-pyplot`

## After publish

- Confirm [docs.rs/plotine](https://docs.rs/plotine) builds
- Tag `vX.Y.Z` and push
- Update `CHANGELOG.md` `[Unreleased]` section
