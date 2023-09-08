use crate::data::*;
use druid::widget::{Button, Flex, Label, Painter, TextBox};
use druid::{
    Color, EventCtx, LocalizedString, RenderContext, Widget, WidgetExt, WindowDesc,
    WindowState, WindowConfig, Env
};
use druid_widget_nursery::DropdownSelect;

pub fn build_ui() -> impl Widget<AppState> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let width = display_info[0].width as f64;
    let height = display_info[0].height as f64;

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
        .with_child(Flex::row().with_child(Button::new("Nuovo").on_click(
            |_ctx, data: &mut AppState, _env| {
                data.screen();
                data.name = "".to_string();
            },
        )))
        .with_child(
            Button::new("Shortcut").on_click(|ctx: &mut EventCtx, _data, _env| {
                let new_win = WindowDesc::new(shortcut_ui())
                    .title(LocalizedString::new("Shortcut"))
                    .window_size((300.0, 200.0));
                ctx.new_window(new_win);
            }),
        )
        .with_child(
            Button::new("Area").on_click(move |ctx: &mut EventCtx, data: &mut AppState, env: &Env| {
                let mut current = ctx.window().clone();
                current.set_window_state(WindowState::Minimized);
                let new_win = WindowDesc::new(drag_motion_ui())
                    .show_titlebar(false)
                    .transparent(true)
                    .window_size((width, height))
                    .resizable(false)
                    .set_position((0.0, 0.0));
                ctx.new_window(new_win);
                //ctx.new_sub_window(WindowConfig::default().transparent(true), drag_motion_ui(), data.clone(), env.clone());
            }),
        )
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

pub fn drag_motion_ui() -> impl Widget<AppState> {
    let paint = Painter::new(|ctx, data: &AppState, _env| {
        if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
            let rect = druid::Rect::from_points(start, end);
            ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
            ctx.stroke(rect, &druid::Color::WHITE, 2.0);
        }
    })
    .controller(PainterController {})
    .controller(AreaController {})
    .center();

    Flex::column().with_child(paint)
}
