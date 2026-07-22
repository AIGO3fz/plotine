# Contributing to plotine

Thanks for helping. plotine prioritizes **correctness and visual review** over feature count.

## Setup

- Rust **1.85+** (MSRV)
- Optional: Python + matplotlib for `scripts/matplotlib_compare.py`
- Optional: `ffmpeg` / Ghostscript / TeX for `mp4` / `eps` / `latex` features

```bash
cargo test -p plotine
cargo run -p plotine --example gallery
```

## Pull requests

1. Prefer small, reviewable PRs.
2. New chart / render change: recipe unit test + gallery entry; Linux visual snapshots via `cargo insta review` only after human eyeballing.
3. Visual constants that encode stock matplotlib behaviour go in `crates/plotine/src/mpl_policy.rs` — do not sprinkle magic floats in recipes.
4. Update `CHANGELOG.md` for user-visible changes.
5. Design changes: edit `docs/DEVELOPMENT_PLAN.md` first.

## API style

Follow `AGENTS.md`: `Figure::new().axes(|ax| { … }).save(…)` — not global pyplot state in the main crate.

## Discussions

Use GitHub Issues for bugs / feature requests. For open-ended design talk, open a Discussion (or an Issue labeled `design`) and link `docs/MPL_GAP.md` when the topic is matplotlib parity.

## Community

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md) — report vulnerabilities privately, not via public Issues

## License

By contributing you agree your work is under the MIT license (see `LICENSE`).
