use iced::Theme;

pub fn rounded_box(theme: &Theme) -> iced::widget::container::Appearance {
    let palette = theme.extended_palette();

    iced::widget::container::Appearance  {
        background: Some(palette.background.weak.color.into()),
        ..iced::widget::container::Appearance ::default()
    }
}
