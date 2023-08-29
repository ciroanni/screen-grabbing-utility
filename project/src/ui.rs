use crate::data::*;
use druid::widget::{Button, Flex, Label, TextBox};
use druid::{EventCtx, Widget, WidgetExt, LocalizedString, WindowDesc};
use druid_widget_nursery::DropdownSelect;

pub fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            TextBox::new()
                .with_placeholder("Insert your screenshot name")
                .expand_width()
                .lens(AppState::name)
                .controller(Enter {}),
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
            |_ctx, data: &mut AppState, _env| {
                data.screen();
                data.name = "".to_string();
            },
        )))
        .with_child(Button::new("Shortcut").on_click(
            |ctx: &mut EventCtx, _data, _env| {
                let new_win = WindowDesc::new(shortcut_ui())
                    .title(LocalizedString::new("Shortcut"))
                    .window_size((300.0, 200.0));
                ctx.new_window(new_win);
            },
        ))
}

pub fn shortcut_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(Label::new("Insert your shortcut:"))
        .with_child(
            TextBox::new()
                .with_placeholder("es. press ALT+S")
                .expand_width()
                .lens(AppState::shortcut)
                .controller(ShortcutController {}),
        )
}
