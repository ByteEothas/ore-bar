use iced::widget::container;
use iced::Theme;
pub fn pane_pop(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        ..Default::default()
    }
}
