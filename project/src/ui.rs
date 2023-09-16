use crate::data::*;
use arboard::{Clipboard, ImageData};
use druid::widget::{Button, FillStrat, Flex, Image, Label, Painter, SizedBox, TextBox};
use druid::{
    commands, Color, Env, EventCtx, FileDialogOptions, FileSpec, ImageBuf,
    LocalizedString, Menu, MenuItem, RenderContext, Widget, WidgetExt,
    WindowDesc, WindowId, WindowState,
};
use druid_shell::TimerToken;
use druid_widget_nursery::DropdownSelect;
use std::borrow::Cow;

pub fn build_ui() -> impl Widget<AppState> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let width = display_info[0].width as f64;
    let height = display_info[0].height as f64;

    Flex::column()
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
        .with_child(Button::new("Nuovo").on_click(
            move |ctx: &mut EventCtx, _data, _env| {
                let mut current = ctx.window().clone();
                current.set_window_state(WindowState::Minimized);
                let new_win = WindowDesc::new(drag_motion_ui(true))
                    .show_titlebar(false)
                    .transparent(true)
                    .window_size((width, height))
                    .resizable(false)
                    .set_position((0.0, 0.0));
                ctx.new_window(new_win);
            },
        ))
        .with_child(Button::new("Area").on_click(
            move |ctx: &mut EventCtx, _data: &mut AppState, _env: &Env| {
                let mut current = ctx.window().clone();
                current.set_window_state(WindowState::Minimized);
                let new_win = WindowDesc::new(drag_motion_ui(false))
                    .show_titlebar(false)
                    .transparent(true)
                    .window_size((width, height))
                    .resizable(false)
                    .set_position((0.0, 0.0));
                ctx.new_window(new_win);
            },
        ))
        .controller(Enter {})
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

pub fn drag_motion_ui(is_full: bool) -> impl Widget<AppState> {
    let paint = Painter::new(|ctx, data: &AppState, _env| {
        if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
            let rect = druid::Rect::from_points(start, end);
            ctx.fill(
                rect,
                &Color::rgba(0.0, 0.0, 0.0, data.selection_transparency),
            );
            //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
        }
    })
    .controller(AreaController {
        id_t: TimerToken::next(),
        id_t2: TimerToken::next(),
        flag: is_full,
    })
    .center();

    Flex::column().with_child(paint)
}

pub fn show_screen_ui(img: ImageBuf) -> impl Widget<AppState> {
    let image = Image::new(img).fill_mode(FillStrat::ScaleDown);
    Flex::column()
        .with_child(
            Button::new("Resize").on_click(|ctx: &mut EventCtx, _data, _env| {
                let paint = Painter::new(|ctx, data: &AppState, _env| {
                    if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
                        let rect = druid::Rect::from_points(start, end);
                        ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
                        //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
                    }
                })
                .controller(ResizeController {});
                let mut current = ctx.window().clone();
                current.set_window_state(WindowState::Minimized);
                let new_win = WindowDesc::new(
                    Flex::column()
                        .with_child(
                            Button::new("Ok").on_click(|ctx: &mut EventCtx, _data, _env| {}),
                        )
                        .with_child(
                            Button::new("Annulla").on_click(|ctx: &mut EventCtx, _data, _env| {}),
                        )
                        .with_child(paint),
                )
                .show_titlebar(false)
                .transparent(true)
                .window_size((1980., 1020.))
                .resizable(false)
                .set_position((0.0, 0.0));
                ctx.new_window(new_win);
            }),
        )
        .with_child(SizedBox::new(image).width(500.).height(500.))
}

#[allow(unused_assignments)]
pub fn make_menu(_: Option<WindowId>, _state: &AppState, _: &Env) -> Menu<AppState> {
    let save_dialog = FileDialogOptions::new()
        .allowed_types(formats())
        .default_type(FileSpec::JPG)
        .default_name("screenshot")
        .name_label("Target")
        .title("Choose a target for this lovely file")
        .button_text("Export");

    let open_dialog = FileDialogOptions::new()
        .select_directories()
        .button_text("Import");
    let base = Menu::empty();
    let mut file = Menu::new(LocalizedString::new("File"));
    file = file
        .entry(
            MenuItem::new(LocalizedString::new("Save"))
                .on_activate(
                    //salvo nel path di default
                    |_ctx, data: &mut AppState, _env| {
                        data.save();
                    },
                )
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty()),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Save as")) //posso scegliere il path
                .command(commands::SHOW_SAVE_PANEL.with(save_dialog))
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty()),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Copy"))
                .on_activate(move |_ctx, data: &mut AppState, _env| {
                    //ctx.submit_command(commands::SHOW_SAVE_PANEL.with(save_dialog_options.clone()))
                    let img = ImageData {
                        width: data.img.width(),
                        height: data.img.height(),
                        bytes: Cow::from(data.img.raw_pixels()),
                    };
                    let mut clip = Clipboard::new().unwrap();
                    clip.set_image(img).unwrap();
                })
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty()),
        );

    let mut modifica = Menu::new(LocalizedString::new("Modify"));
    modifica = modifica
        .entry(
            MenuItem::new(LocalizedString::new("Save in")) //mi permette di scegliere il path di default in cui salva premendo SAVE
                .command(commands::SHOW_OPEN_PANEL.with(open_dialog)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Shortcut"))
                .on_activate(move |ctx, _data, _env| ctx.submit_command(SHORTCUT)),
        );

    /* let modify = Menu::new(LocalizedString::new("Modifica"));
       modify = modify.entry(MenuItem::new(LocalizedString::new("Ritaglia")).on_activate(
           |ctx: &mut EventCtx, _data, _env| {
               let paint = Painter::new(|ctx, data: &AppState, _env| {
                   if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
                       let rect = druid::Rect::from_points(start, end);
                       ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
                       //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
                   }
               });
               let mut current = ctx.window().clone();
               current.set_window_state(WindowState::Minimized);
               let new_win = WindowDesc::new(
                   Flex::column()
                       .with_child(paint)
                       .controller(ResizeController {}),
               )
               .show_titlebar(false)
               .transparent(true)
               .window_size((2560., 1600.))
               .resizable(false)
               .set_position((0.0, 0.0));
               ctx.new_window(new_win);
           },
       ));
    */
    base.entry(file).entry(modifica)
}

pub fn formats() -> Vec<FileSpec> {
    vec![
        FileSpec::JPG,
        FileSpec::PNG,
        FileSpec::GIF,
        FileSpec::new("Webp", &["webp"]),
        FileSpec::new("Pnm", &["pnm"]),
        FileSpec::new("Tiff", &["tiff"]),
        FileSpec::new("Tga", &["tga"]),
        FileSpec::new("Dds", &["dds"]),
        FileSpec::new("Bmp", &["bmp"]),
        FileSpec::new("Ico", &["ico"]),
        FileSpec::new("Hdr", &["hdr"]),
        FileSpec::new("OpenExr", &["openexr"]),
        FileSpec::new("Farbfeld", &["farbfeld"]),
        FileSpec::new("Avif", &["avif"]),
        FileSpec::new("Qoi", &["qoi"]),
    ]
}
