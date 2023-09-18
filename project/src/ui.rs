use crate::data::*;
use arboard::{Clipboard, ImageData};
use druid::widget::{
    BackgroundBrush, Button, Either, FillStrat, Flex, Image, Label, Painter, SizedBox, TextBox,
    ZStack,
};
use druid::{
    commands, Color, Env, EventCtx, FileDialogOptions, FileSpec, ImageBuf, LocalizedString, Menu,
    MenuItem, RenderContext, Size, UnitPoint, Vec2, Widget, WidgetExt, WidgetPod, WindowDesc,
    WindowId, WindowLevel, WindowState,
};
use druid_shell::keyboard_types::Modifiers;
use druid_shell::TimerToken;
use druid_widget_nursery::DropdownSelect;
use std::borrow::Cow;

pub fn build_ui(img: ImageBuf) -> impl Widget<AppState> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let width = display_info[0].width as f64;
    let height = display_info[0].height as f64;

    let flex_col = Flex::column();
    flex_col
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Nuovo")
                        .on_click(move |ctx: &mut EventCtx, _data, _env| {
                            let current = ctx.window().clone();
                            current.close();
                            let new_win = WindowDesc::new(drag_motion_ui(true))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((width, height))
                                .resizable(false)
                                .set_position((0.0, 0.0));
                            ctx.new_window(new_win);
                        })
                        .fix_width(100.0)
                        .fix_height(30.0),
                )
                .with_spacer(50.)
                .with_child(
                    DropdownSelect::new(vec![
                        ("Nessun Timer", Timer::Zero),
                        ("3 secondi", Timer::ThreeSeconds),
                        ("5 secondi", Timer::FiveSeconds),
                        ("10 secondi", Timer::TenSeconds),
                    ])
                    .fix_width(120.0)
                    .fix_height(30.0)
                    .align_left()
                    .lens(AppState::delay),
                )
                .with_spacer(50.)
                .with_child(
                    Button::new("Area")
                        .on_click(move |ctx: &mut EventCtx, data: &mut AppState, _env: &Env| {
                            data.rect = SelectionRectangle::default();
                            let current = ctx.window().clone();
                            current.close();
                            let new_win = WindowDesc::new(drag_motion_ui(false))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((width, height))
                                .resizable(false)
                                .set_position((0.0, 0.0));
                            ctx.new_window(new_win);
                        })
                        .fix_width(100.0)
                        .fix_height(30.0),
                )
                .controller(Enter {
                    id_t: TimerToken::next(),
                    id_t2: TimerToken::next(),
                }),
        )
        .with_spacer(100.)
        .with_child(Either::new(
            |data: &AppState, _env| data.img.size() == Size::ZERO,
            Label::new(|data: &AppState, _env: &_| {
                format!(
                    "Premi {:?} {} per la cattura",
                    Modifiers::from_bits(data.mods).unwrap_or(Modifiers::empty()),
                    String::from("+ ")
                        + &char::from_u32(data.key).unwrap().to_string().to_uppercase()
                )
            })
            .with_text_size(24.)
            .center(),
            ZStack::new(
                SizedBox::new(Image::new(img.clone()))
                    .width(500.)
                    .height(312.5)
                    .background(BackgroundBrush::Color(druid::Color::rgb(255., 0., 0.))),
            )
            //show_screen_ui(img),
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
    let image = Image::new(img.clone());
    //let brush = BackgroundBrush::Color(druid::Color::rgb(255., 0., 0.));
    let row = Flex::row()
        .with_child(Button::new("Resize").on_click(
            |ctx: &mut EventCtx, data: &mut AppState, _env| {
                let paint = Painter::new(|ctx, data: &AppState, _env| {
                    if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
                        println!("{} {}", start, end);
                        let rect = druid::Rect::from_points(start, end);
                        ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
                        //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
                    }
                })
                .controller(ResizeController {});
                let new_win = WindowDesc::new(
                    //Flex::column()
                    /*.with_child(
                        Button::new("Ok").on_click(|ctx: &mut EventCtx, data: &mut AppState, _env| {
                            data.rect.size = druid::Rect::from_points(data.rect.start_point.unwrap(), data.rect.end_point.unwrap()).size();
                            data.screen(ctx);
                        }),
                    )
                    .with_child(
                        Button::new("Annulla").on_click(|ctx: &mut EventCtx, _data, _env| {
                            ctx.window().close();
                        }),
                    ) */
                    //.with_child(paint)
                    paint.controller(Enter {
                        id_t: TimerToken::next(),
                        id_t2: TimerToken::next(),
                    }),
                )
                .show_titlebar(false)
                .transparent(true)
                .window_size((
                    data.rect.size.width * data.scale as f64,
                    data.rect.size.height * data.scale as f64,
                ))
                .resizable(false)
                .set_position((0., 0.));
                ctx.new_window(new_win);
            },
        ))
        .with_child(Button::new("Modifica Immagine")); //ANNOTATION TOOLS

    ZStack::new(
        SizedBox::new(image)
            .width(500.)
            .height(312.5),
    )
    .with_centered_child(Painter::new(|ctx, data: &AppState, _env| {
        if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
            let rect = druid::Rect::from_points(start, end);
            ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
            //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
        }
    }).controller(ResizeController{}))

    /* .with_child(
        Painter::new(|ctx, data: &AppState, _env| {
            println!("Painter");
            if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
                let rect = druid::Rect::from_points(start, end);
                ctx.fill(rect, &Color::rgba(0.0, 0.0, 0.0, 0.4));
                //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
            }
        })
        .controller(ResizeController {}),
    ) */
}

#[allow(unused_assignments)]
pub fn make_menu(_: Option<WindowId>, _state: &AppState, _: &Env) -> Menu<AppState> {
    let save_dialog = FileDialogOptions::new()
        .allowed_types(formats())
        .default_type(FileSpec::JPG)
        .default_name("screenshot")
        .name_label("Target")
        .button_text("Export");

    let open_dialog = FileDialogOptions::new()
        .select_directories()
        .button_text("Import");
    let base = Menu::empty();
    let mut file = Menu::new(LocalizedString::new("File"));
    file = file
        .entry(
            MenuItem::new(LocalizedString::new("Salva"))
                .on_activate(
                    //salvo nel path di default
                    |_ctx, data: &mut AppState, _env| {
                        data.save();
                    },
                )
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty()),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Salva come")) //posso scegliere il path
                .command(commands::SHOW_SAVE_PANEL.with(save_dialog))
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty()),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Copia"))
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

    let mut modifica = Menu::new(LocalizedString::new("Modifica"));
    modifica = modifica
        .entry(
            MenuItem::new(LocalizedString::new("Salva in")) //mi permette di scegliere il path di default in cui salva premendo SAVE
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
