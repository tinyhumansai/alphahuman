# Vendored TokenJuice Rules

These JSON rule files are vendored from the upstream
[vincentkoc/tokenjuice](https://github.com/vincentkoc/tokenjuice) repository.

## Upstream

- Repository: https://github.com/vincentkoc/tokenjuice
- Upstream path: `src/rules/**/*.json`
- Licence: MIT (Copyright (c) 2026 Vincent Koc)

## Licence note

The upstream project is MIT-licensed. The full licence text is reproduced below.

```
MIT License

Copyright (c) 2026 Vincent Koc

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## File naming convention

Upstream files live in subdirectory paths like `git/status.json`.  Because we
embed all rules in a single directory here, `/` in the id is replaced with `__`
in the filename (e.g. `git/status.json` → `git__status.json`).

## Rules vendored

**96 rules** are vendored here, representing the complete set of generic rules
from the upstream repository as of 2026-04-17.

### Exclusions

The `src/rules/openclaw/` subdirectory in upstream is **not** vendored.  Those
rules (`openclaw/sessions-history`, etc.) are specific to the upstream author's
proprietary OpenClaw tooling and are not generic enough to include in the
OpenHuman builtin set.  The `fixtures/` subdirectory is also excluded — fixture
files are test-only and carry no runtime behaviour.

### Adding more rules

Additional rules from the upstream repository can be added by:

1. Copying the JSON verbatim into this directory using the `family__name.json`
   naming convention.
2. Adding the corresponding `(id, include_str!(...))` entry to
   `rules/builtin.rs`, keeping the list alphabetically ordered by id.
3. Running `cargo check` and `cargo test tokenjuice` to confirm the new rule
   compiles cleanly.
