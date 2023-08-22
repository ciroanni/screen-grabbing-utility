use crate::data::*;
use druid::widget::{Align, Button, Flex, Label, Radio, RadioGroup, TextBox, CrossAxisAlignment};
use druid::PlatformError;
use druid::{
    im::Vector, AppLauncher, Data, Lens, LocalizedString, UnitPoint, Widget, WidgetExt, WindowDesc,
};

pub fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            TextBox::new()
                .with_placeholder("es. screen.jpeg")
                .expand_width()
                .lens(AppState::name),
        )
        .with_spacer(20.0)
        .with_child(
            Button::new("+ Nuovo").on_click(|ctx, data: &mut AppState, _env| {
                data.screen();
            }),
        )
        .with_child(
            Radio::new("Jpeg", ImageFormat::Jpeg)
                .on_click(|ctx, data: &mut ImageFormat, _| *data = ImageFormat::Jpeg)
                .lens(AppState::selected_format),
        )
        .with_child(
            Radio::new("Png", ImageFormat::Png)
                .on_click(|ctx, data: &mut ImageFormat, _| *data = ImageFormat::Png)
                .lens(AppState::selected_format),
        )
        .with_child(
            Radio::new("Gif", ImageFormat::Gif)
                .on_click(|ctx, data: &mut ImageFormat, _| *data = ImageFormat::Gif)
                .lens(AppState::selected_format),
        )
}
