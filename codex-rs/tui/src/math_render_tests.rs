use super::normalize_for_terminal;
use super::render_display_math;
use super::render_inline_math;

#[test]
fn renders_single_line_inline_math() {
    assert_eq!(
        render_inline_math(r"\alpha + \beta \leq \gamma", Some(80)),
        Some("α + β ≤ γ".to_string())
    );
}

#[test]
fn leaves_multiline_inline_math_for_source_fallback() {
    assert_eq!(render_inline_math(r"\frac{a}{b}", Some(80)), None);
}

#[test]
fn renders_display_fraction() {
    assert_eq!(
        render_display_math(r"\frac{a}{b}", Some(80)),
        Some(vec![" a".to_string(), "───".to_string(), " b".to_string()])
    );
}

#[test]
fn renders_operatorname_with_terminal_compatible_upright_text() {
    assert_eq!(
        render_display_math(r"f(x)=\operatorname{ReLU}(xW_1+b_1)W_2+b_2", Some(80)),
        Some(vec!["f(x) = ReLU(xW₁ + b₁)W₂ + b₂".to_string()])
    );
}

#[test]
fn renders_sized_delimiters_in_rank_bound() {
    assert_eq!(
        render_display_math(
            r"\operatorname{rank}(W')
\leq
\min\bigl(
\operatorname{rank}(W_1),
\operatorname{rank}(W_2)
\bigr)
\leq H",
            Some(80)
        ),
        Some(vec!["rank(W') ≤ min( rank(W₁), rank(W₂) ) ≤ H".to_string()])
    );
}

#[test]
fn renders_outer_box_around_text() {
    assert_eq!(
        render_display_math(r"\boxed{\text{能表示什么函数？ }}", Some(80)),
        Some(vec![
            "┌─────────────────┐".to_string(),
            "│能表示什么函数？ │".to_string(),
            "└─────────────────┘".to_string(),
        ])
    );
}

#[test]
fn normalizes_only_complete_latex_command_names() {
    assert_eq!(render_display_math(r"\bmatrix", Some(80)), None);
}

#[test]
fn normalizes_common_unsupported_presentation_commands() {
    assert_eq!(
        normalize_for_terminal(
            r"\displaystyle \operatorname*{argmax}\limits_x \dfrac{a}{b} + \boldsymbol{v} + \Bigl(x\Bigr)"
        ),
        r"\mathrm{argmax}_x \frac{a}{b} + \mathbf{v} + (x)"
    );
}

#[test]
fn collapses_source_line_breaks_without_joining_tokens() {
    assert_eq!(normalize_for_terminal("x\n+\n y\t=  z"), "x + y = z");
}

#[test]
fn rejects_display_math_wider_than_the_viewport() {
    assert_eq!(
        render_display_math(r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}", Some(8)),
        None
    );
}

#[test]
fn rejects_unknown_empty_environment() {
    assert_eq!(
        render_display_math(r"\begin{unknown}x\end{unknown}", Some(80)),
        None
    );
}
