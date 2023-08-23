use crate::data::*;
use druid::widget::{Align, Button, CrossAxisAlignment, Flex, Label, Radio, RadioGroup, TextBox};
use druid::PlatformError;
use druid::{
    im::Vector, AppLauncher, Data, Env, EventCtx, Lens, LocalizedString, UnitPoint, Widget,
    WidgetExt, WindowDesc,
};
use druid_widget_nursery::dropdown::DROPDOWN_SHOW;
use druid_widget_nursery::{Dropdown, DropdownSelect};

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
            DropdownSelect::new(vec![
                ("Jpeg", ImageFormat::Jpeg),
                ("Png", ImageFormat::Png),
                ("Gif", ImageFormat::Gif),
            ])
            .align_left()
            .lens(AppState::selected_format),
        )
        .with_child(Flex::row().with_child(Button::new("+ Nuovo").on_click(
            |ctx, data: &mut AppState, _env| {
                data.screen();
            },
        )))
}
