# Terminal LaTeX rendering

This fork teaches the Codex terminal UI to recognize Markdown math and render supported LaTeX as Unicode character art directly in the transcript.

## Supported syntax

Inline math uses single dollar delimiters:

```markdown
Energy: $E = mc^2$
```

Display math uses double dollar delimiters:

```markdown
$$\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

$$\begin{bmatrix}1 & 2 \\ 3 & 4\end{bmatrix}$$
```

The terminal output uses Unicode symbols and multi-line character art. Math inside code spans and fenced code blocks remains literal.

## Safety and fallback behavior

Rendering is best effort. Unsupported expressions, multi-line inline expressions, excessively large input, and output wider than the current transcript width fall back to the original Markdown source. This keeps answers readable and preserves the LaTeX for copying or transcript export.

## Build and run

On a system with the repository's Rust toolchain installed:

```shell
cd codex-rs
cargo build -p codex-cli --bin codex
./target/debug/codex --no-alt-screen
```

The custom binary uses the same Codex home directory, authentication, and configuration as the standard CLI unless you override them explicitly.

## Implementation

- `pulldown-cmark` identifies inline and display math while parsing Markdown.
- `term-maths` renders the LaTeX payload as bounded Unicode output.
- The streaming renderer enables the same math extension, so a completed streamed response matches the full transcript render.
- Panic, size, height, width, and unsupported-output guards preserve the original source as a fallback.

This repository remains licensed under the upstream [Apache-2.0 License](../LICENSE).
