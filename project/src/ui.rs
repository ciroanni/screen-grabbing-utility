use crate::data::*;
use druid::widget::{Button, Flex, Label, Radio, RadioGroup, TextBox};
use druid::PlatformError;
use druid::{im::Vector, AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};
use image::ImageFormat;

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
}