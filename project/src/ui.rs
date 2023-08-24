use crate::data::*;
use druid::widget::{Align, Button, CrossAxisAlignment, Flex, Label, Radio, RadioGroup, TextBox};
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
                .with_placeholder("es. screenshot.jpeg")
                .expand_width()
                .lens(AppState::name),
        )
        .with_spacer(20.0)
        .with_child(
            DropdownSelect::new(vec![
                ("Jpeg", ImageFormat::Jpeg),
                ("Png", ImageFormat::Png),
                ("Gif", ImageFormat::Gif),
                ("Webp", ImageFormat::WebP),
                ("Pnm", ImageFormat::Pnm),
                ("Tiff", ImageFormat::Tiff),
                ("Tga", ImageFormat::Tga),
                ("Dds", ImageFormat::Dds),
                ("Bmp", ImageFormat::Bmp),
                ("Ico", ImageFormat::Ico),
                ("Hdr", ImageFormat::Hdr),
                ("OpenExr", ImageFormat::OpenExr),
                ("Farbfeld", ImageFormat::Farbfeld),
                ("Avif", ImageFormat::Avif),
                ("Qoi", ImageFormat::Qoi),
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
