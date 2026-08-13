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
