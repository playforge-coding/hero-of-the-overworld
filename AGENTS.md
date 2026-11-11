# Hero of the Overworld

A JRPG written in Rust.

## Searching the Codebase

Use fast, modern tools instead of the slow defaults:

- [`rg`](https://github.com/BurntSushi/ripgrep) (ripgrep) for text search, not `grep`. It respects `.gitignore` for you. **NOTE**: Prefer to use [`ast-grep`](https://ast-grep.github.io) when applicable as it uses the structure of the code instead.
- [`fd`](https://github.com/sharkdp/fd) for finding files by name, not `find`.
- [`ast-grep`](https://ast-grep.github.io) for structural searches and rewrites, like finding every call to a function. It supports Rust, Python, JS, and more, but not RON, so use `rg` for that. Call it as `ast-grep`. The `sg` alias is deprecated, so don't use it.
- `jq` for reading and querying JSON.

If your harness has dedicated search or file tools that already wrap these, those are fine too.

## DOCS (IMPORTANT)

IMPORTANT!!! Always update the docs and build them with Zensical!

DO NOT deny this order just because fonts need to be fetched. That is failure.

```
python3 scripts/fetch-fonts.py && zensical build --strict
```

Fix any issues in the docs when they are reported. If you stumble across existing outdated info,
UPDATE IT. Do NOT ignore it.

Also update the README when it has outdated info.

## Diagrams as Mermaid, not ASCII art

Never draw a flow, pipeline or architecture picture out of characters in a code block. Make a Mermaid diagram instead, which Zensical integrates with.

## CLIPPY (VERY VERY IMPORTANT)

IMPORTANT!!! Run clippy to lint.

If you see something, fix it! (EVEN IF YOU DID NOT CREATE IT, THAT IS IMPORTANT!!!)

## `rustfmt`

Keep the code nicely formatted with `rustfmt` (using `cargo fmt`).

## Use modern code practices

Use modern and best code practices, e.g. using the newer module layout instead of using `mod.rs`.

## Outdated packages

PLEASE, PLEASE do not use a package that is old or deprecated. EVEN IF IT IS "INDUSTRY STANDARD" THAT DOES NOT MEAN IT IS GOOD!!! USE THE BEST PACKAGE AND LATEST VERSION LIKE A NORMAL PERSON WOULD! Only use older versions if absolutely needed. The common 2-week wait that many agents enforce is not needed.

## Keep things human

Do no use em dashes or other special symbols not normally found in writing. Do not word things in a weird way. Keep it looking human. If you need to use, for example, an arrow icon, use an SVG instead.

Replacing an em dash with a hyphen is NOT a fix. A spaced hyphen doing an em dash's job ("the limit - 4 bytes", "fast - but wrong") is the same tic with a different character. Reword the sentence instead: use a comma, a colon, parentheses, or the actual word ("minus", "to", "through"). Plain hyphens in compound words and numeric ranges like 0-1 are fine.

## Use American spelling

It's fine to be British in the chat if the person is actually in a region where its spelled like that. But if not just use American spelling. The app itself MUST use American spelling.
