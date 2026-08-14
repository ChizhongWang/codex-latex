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

The common LaTeX forms `\(...\)` and `\[...\]` are supported as well. Multi-line display
expressions are folded before CommonMark parsing so a standalone `=` cannot be mistaken for a
Setext heading. The rewrite preserves byte offsets used by streaming and resume rendering.

The terminal output uses Unicode symbols and multi-line character art. Math inside code spans and fenced code blocks remains literal.

## Safety and fallback behavior

Rendering is best effort. Unsupported expressions, multi-line inline expressions, excessively
large input, and output wider than the current transcript width fall back to compact,
delimiter-free LaTeX source. Source line breaks are folded so one unsupported display expression
cannot expand into a page of scattered `\[`/`\]` fragments, while the formula remains available
for copying and diagnosis.

## Build and run

On a system with the repository's Rust toolchain installed:

```shell
cd codex-rs
cargo build -p codex-cli --bin codex
./target/debug/codex --no-alt-screen
```

The custom binary uses the same Codex home directory, authentication, and configuration as the standard CLI unless you override them explicitly.

## Implementation

- A byte-length-preserving compatibility pass recognizes `$...$`, `$$...$$`, `\(...\)`, and
  `\[...\]`, protects code spans and fences, and removes Markdown block ambiguities.
- `pulldown-cmark` then identifies inline and display math while parsing Markdown.
- A compatibility layer translates standard presentation commands and delimiter spellings into
  the subset accepted by `term-maths`, which renders bounded Unicode output.
- The streaming renderer enables the same math extension, so a completed streamed response matches the full transcript render.
- Panic, size, height, width, and unsupported-command guards switch the whole expression to the
  compact source fallback instead of leaking a partially rendered formula.

The Codex desktop app was used as a behavioral reference, not as a code dependency. Its WebView
renderer uses KaTeX with `strict: "ignore"` and `throwOnError: false`; this fork follows the same
graceful-degradation policy. KaTeX's DOM and font output cannot be inserted directly into a
terminal transcript, so supported formulas use Unicode and unsupported formulas retain readable
source. Pixel/image math remains a separate optional backend because Kitty, Sixel, and iTerm image
protocols do not provide portable scrollback semantics across terminals.

This repository remains licensed under the upstream [Apache-2.0 License](../LICENSE).
