use crate::data::*;
use druid::widget::{Align, Button, Flex, Label, Radio, RadioGroup, TextBox, CrossAxisAlignment};
use druid::PlatformError;
use druid::{
    im::Vector, AppLauncher, Data, Lens, LocalizedString, UnitPoint, Widget, WidgetExt, WindowDesc,
};
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
        .with_child(radiogroup())
}

pub fn radiogroup() -> impl Widget<AppState> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Radio::new(
            "Jpeg",
            AppState {
                name: "".to_string(),
                format: ".jpeg".to_string(),
            },
        ))
        .with_child(Radio::new(
            "Png",
            AppState {
                name: "".to_string(),
                format: ".png".to_string(),
            },
        ))
        .with_child(Radio::new(
            "Gif",
            AppState {
                name: "".to_string(),
                format: ".gif".to_string(),
            },
        ))
}
