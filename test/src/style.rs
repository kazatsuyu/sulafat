#[cfg(test)]
mod test {
    use sulafat_macros::StyleSet;
    use sulafat_style::{
        CSSRenderer, Length, LengthOrPercentage, Parcentage, StyleRenderer, StyleRule, StyleSet,
        WritingMode,
    };

    #[derive(StyleSet)]
    #[style_set{
        .test {
            left: 100px;
            right: 100 %;
            writing - mode: vertical - rl;
        }
    }]
    struct Style;

    #[test]
    fn it_works() {
        assert_eq!(
            Style::rules(),
            &[
                StyleRule::Left(LengthOrPercentage::Length(Length::Px(100.))),
                StyleRule::Right(LengthOrPercentage::Parcentage(Parcentage(100.))),
                StyleRule::WritingMode(WritingMode::VerticalRl),
            ]
        )
    }

    #[test]
    fn css_renderer() {
        let mut renderer = CSSRenderer::default();
        renderer.name(&Style::name());
        Style::render(&mut renderer);
        assert_eq!(
            renderer.finish(),
            ".test{left:100px;right:100%;writing-mode:vertical-rl;}"
        );
    }
}
