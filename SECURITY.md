# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.5.x   | ✅        |
| < 0.5   | ❌        |

Only the latest published minor line on crates.io receives security fixes.

## Reporting a vulnerability

Please **do not** open a public GitHub Issue for security vulnerabilities.

Report privately via one of:

1. [GitHub Security Advisories](https://github.com/AIGO3fz/plotine/security/advisories/new)
   (preferred when the repository is public)
2. Contact the maintainer **[@AIGO3fz](https://github.com/AIGO3fz)** on GitHub

Include:

- Affected crate(s) and versions
- A clear description of the issue and impact
- Steps to reproduce or a proof of concept (if available)
- Whether you are aware of active exploitation

## Response expectations

- Acknowledgement within **7 days** (best effort)
- Status update within **14 days** after acknowledgement
- Coordinated disclosure: please give us time to publish a fix before public
  discussion, unless you have already disclosed the issue elsewhere

## Scope notes

plotine is a plotting library. Typical security-relevant surfaces include:

- Path handling in `.save` / animation export / optional external tools
  (`ffmpeg`, Ghostscript, `latex`/`dvipng`)
- Parsing of untrusted GeoJSON or similar input when those APIs are used
- Dependency vulnerabilities in the Rust crate graph

Issues that only affect visual output (wrong pixels, layout) are bugs — file a
normal Issue, not a security report.
