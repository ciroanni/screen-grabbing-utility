use crate::data::*;
use arboard::{Clipboard, ImageData};
use druid::widget::{
    BackgroundBrush, Button, Either, Flex, Image, Label, Painter, SizedBox,
    Switch, TextBox, ViewSwitcher, ZStack,Container
};
use druid::{
    commands, Color, Env, EventCtx, FileDialogOptions, FileSpec, ImageBuf, LocalizedString, Menu,
    MenuItem, Point, RenderContext, Size, UnitPoint, Widget, WidgetExt,
    WindowDesc, WindowId
};
use druid_shell::{SysMods, TimerToken, HotKey};
use druid_widget_nursery::{DropdownSelect, WidgetExt as _};
use image::{GenericImage, ImageBuffer, Rgba};
use rusttype::Font;
use std::borrow::Cow;
use std::str::FromStr;

pub fn build_ui(scale: f32) -> impl Widget<AppState> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");
    //let scale = display_info[0].scale_factor;
    let mut width = (display_info[0].width as f32 * display_info[0].scale_factor) as u32;
    let mut height = (display_info[0].height as f32 * display_info[0].scale_factor) as u32;
    let mut pos = Point::new(0., 0.);
    let hotkey=HotKey::new(Some(druid_shell::RawMods::Alt), 'g'.to_string().as_str());

    for display in display_info.iter() {
        if display.x < 0 {
            if display.x + display.width as i32 == 0 {
                width += (display.width as f32 * display.scale_factor) as u32;
            } else {
                width = (width as i32 - display.x) as u32
            }
            pos.x = ((display.x as f32 / scale) * display.scale_factor as f32) as f64;
        } else if display.x as f32 / scale >= display_info[0].width as f32 {
            width += (display.width as f32 * display.scale_factor) as u32;
        } else {
            if (display.x as f32 / scale) + (display.width as f32 / scale)
                > display_info[0].width as f32
            {
                width += (display.width as f32 * display.scale_factor) as u32
                    - (display_info[0].width as f32 * scale - display.x as f32) as u32;
            }
        }

        if display.y < 0 {
            if display.y + display.height as i32 == 0 {
                height += (display.height as f32 * display.scale_factor) as u32;
            } else {
                height = (height as i32 - display.y) as u32
            }
            pos.y = ((display.y as f32 / scale) * display.scale_factor as f32) as f64;
        } else if display.y as f32 / scale >= display_info[0].height as f32 {
            height += (display.height as f32 * display.scale_factor) as u32;
        } else {
            if (display.y as f32 / scale) + (display.height as f32 / scale)
                > display_info[0].height as f32
            {
                height += (display.height as f32 * display.scale_factor) as u32
                    - (display_info[0].height as f32 * scale - display.y as f32) as u32;
            }
        }
    }

    let flex_col = Flex::column();
    flex_col
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Nuovo")
                        .on_click(move |ctx: &mut EventCtx, data: &mut AppState, _env| {
                            let current = ctx.window().clone();
                            current.close();
                            data.rect.start_point = Some(Point::new(0., 0.));
                            data.rect.end_point = Some(data.size);
                            data.rect.p2 = Some(Point::new(data.size.x, 0.));
                            data.rect.p3 = Some(Point::new(0., data.size.y));
                            let new_win = WindowDesc::new(drag_motion_ui(true))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((width as f64, height as f64))
                                .resizable(false)
                                .set_position(pos);
                            ctx.new_window(new_win);
                            data.tool_window=AnnotationTools::default();
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
                                .window_size((width as f64, height as f64))
                                .resizable(false)
                                .set_position(pos);
                            ctx.new_window(new_win);
                            data.tool_window=AnnotationTools::default();                            
                        })
                        .fix_width(100.0)
                        .fix_height(30.0),
                )
                .controller(Enter {
                    id_t: TimerToken::next(),
                    id_t2: TimerToken::next(),
                    locks: [false; 5],
                    do_screen:false,
                    witch_screen:1,
                    display: Some(display_info[0]),
                    hotkey
                }),
        )
        .with_spacer(50.)
        .with_child(Either::new(
            |data: &AppState, _env| data.img.size() == Size::ZERO,
            Flex::column()
            .with_child(
                Either::new(|data: &AppState, _env| data.full_mod1.modifier==livesplit_hotkey::Modifiers::empty(),
                    Label::new(|data: &AppState, _env: &_| {
                        format!(
                            "Premi {} per la cattura a schermo intero",
                            data.full_k
                        )
                    })
                    .with_text_size(24.)
                    .center(),
                    Either::new(|data: &AppState, _env| data.full_mod2.modifier==livesplit_hotkey::Modifiers::empty(),
                        Label::new(|data: &AppState, _env: &_| {
                            format!(
                                "Premi {}+{} per la cattura a schermo intero",
                                data.full_mod1.modifier,
                                data.full_k
                            )
                        })
                        .with_text_size(24.)
                        .center(),
                        Either::new(|data: &AppState, _env| data.full_mod3.modifier==livesplit_hotkey::Modifiers::empty(),
                            Label::new(|data: &AppState, _env: &_| {
                                format!(
                                    "Premi {}+{}+{} per la cattura a schermo intero",
                                    data.full_mod1.modifier,
                                    data.full_mod2.modifier,
                                    data.full_k
                                )
                            })
                            .with_text_size(24.)
                            .center(),
                            Label::new(|data: &AppState, _env: &_| {
                                format!(
                                    "Premi {}+{}+{}+{} per la cattura a schermo intero",
                                    data.full_mod1.modifier,
                                    data.full_mod2.modifier,
                                    data.full_mod3.modifier,
                                    data.full_k
                                )
                            })
                            .with_text_size(24.)
                            .center()
                        )
                    )
                )
            )
            .with_child(
                Either::new(|data: &AppState, _env| data.area_mod1.modifier==livesplit_hotkey::Modifiers::empty(),
                    Label::new(|data: &AppState, _env: &_| {
                        format!(
                            "Premi {} per la cattura ad area",
                            data.area_k
                        )
                    })
                    .with_text_size(24.)
                    .center(),
                    Either::new(|data: &AppState, _env| data.area_mod2.modifier==livesplit_hotkey::Modifiers::empty(),
                        Label::new(|data: &AppState, _env: &_| {
                            format!(
                                "Premi {}+{} per la cattura ad area",
                                data.area_mod1.modifier,
                                data.area_k
                            )
                        })
                        .with_text_size(24.)
                        .center(),
                        Either::new(|data: &AppState, _env| data.area_mod3.modifier==livesplit_hotkey::Modifiers::empty(),
                            Label::new(|data: &AppState, _env: &_| {
                                format!(
                                    "Premi {}+{}+{} per la cattura ad area",
                                    data.area_mod1.modifier,
                                    data.area_mod2.modifier,
                                    data.area_k
                                )
                            })
                            .with_text_size(24.)
                            .center(),
                            Label::new(|data: &AppState, _env: &_| {
                                format!(
                                    "Premi {}+{}+{}+{} per la cattura ad area",
                                    data.area_mod1.modifier,
                                    data.area_mod2.modifier,
                                    data.area_mod3.modifier,
                                    data.area_k
                                )
                            })
                            .with_text_size(24.)
                            .center()
                        )
                    )
                )
            )
            ,
            show_screen_ui(),
        ))
}

pub fn shortcut_ui() -> impl Widget<AppState> {
    /*Flex::column()
        .with_child(Label::new("Premi una nuova combinazione"))
        .controller(ShortcutController { locks: [false; 5] })
    */
    Flex::column()
        .with_child(
            Label::new("Full_screen:").align_left()
        )
        .with_child(
            Container::new(
                Flex::row()
                .with_child(
                    Label::new("modifier 1:")
                )
                .with_child(
                    DropdownSelect::new(
                        vec![
                        ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                        ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                        ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                        ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                    ])
                    .fix_width(70.0)
                    .fix_height(30.0)
                    .align_left()
                    .lens(AppState::full_mod1)
                )
                .with_child(Either::new(|data: &AppState, _env| data.full_mod1.modifier != livesplit_hotkey::Modifiers::empty(), 
                    Flex::row()
                    .with_child(Label::new("modifier 2:"))
                    .with_child(
                        DropdownSelect::new(
                            vec![
                            ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                            ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                            ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                            ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                        ])
                        .fix_width(70.0)
                        .fix_height(30.0)
                        .align_left()
                        .lens(AppState::full_mod2)
                    )
                    .with_child(Either::new(|data: &AppState, _env| data.full_mod2.modifier != livesplit_hotkey::Modifiers::empty(),
                        Flex::row()
                        .with_child(Label::new("modifier 3:"))
                        .with_child(
                            DropdownSelect::new(
                                vec![
                                ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                                ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                                ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                                ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                            ])
                            .fix_width(70.0)
                            .fix_height(30.0)
                            .align_left()
                            .lens(AppState::full_mod3)
                        )
                        ,Label::new("")))
                    ,Label::new("")
                ))
                .with_child(
                    Label::new("character:")
                )
                .with_child(
                    TextBox::new().lens(AppState::full_k)
                )
            ) 
            .align_left()
        )
        .with_child(
            Label::new("Drag and Drop:").align_left()
        )
        .with_child(
            Container::new(
                Flex::row()
                .with_child(
                    Label::new("modifier 1:")
                )
                .with_child(
                    DropdownSelect::new(
                        vec![
                        ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                        ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                        ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                        ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                    ])
                    .fix_width(70.0)
                    .fix_height(30.0)
                    .align_left()
                    .lens(AppState::area_mod1)
                )
                .with_child(Either::new(|data: &AppState, _env| data.area_mod1.modifier != livesplit_hotkey::Modifiers::empty(), 
                    Flex::row()
                    .with_child(Label::new("modifier 2:"))
                    .with_child(
                        DropdownSelect::new(
                            vec![
                            ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                            ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                            ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                            ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                        ])
                        .fix_width(70.0)
                        .fix_height(30.0)
                        .align_left()
                        .lens(AppState::area_mod2)
                    )
                    .with_child(Either::new(|data: &AppState, _env| data.area_mod2.modifier != livesplit_hotkey::Modifiers::empty(),
                        Flex::row()
                        .with_child(Label::new("modifier 3:"))
                        .with_child(
                            DropdownSelect::new(
                                vec![
                                ("-", MyModifier{modifier:livesplit_hotkey::Modifiers::empty()}),
                                ("ALT", MyModifier{modifier:livesplit_hotkey::Modifiers::ALT}),
                                ("CTRL", MyModifier{modifier:livesplit_hotkey::Modifiers::CONTROL}),
                                ("SHIFT", MyModifier{modifier:livesplit_hotkey::Modifiers::SHIFT}),
                            ])
                            .fix_width(70.0)
                            .fix_height(30.0)
                            .align_left()
                            .lens(AppState::area_mod3)
                        )
                        ,Label::new("")))
                    ,Label::new("")
                ))
                .with_child(
                    Label::new("character:")
                )
                .with_child(
                    TextBox::new().lens(AppState::area_k)
                )
            ) 
            .align_left()
        )
        .with_child(
            Container::new(
                Flex::column()
                .with_child(Either::new(|data: &AppState, _env| data.err, 
                    Label::new("Shortcut non valida")
                    .with_text_color(Color::RED)
                    , Label::new("")))
                .with_child(
                Flex::row()
                .with_child(
                    Button::new("Salva")
                    .on_click(|ctx,data: &mut AppState,_env|{
                        let res1=livesplit_hotkey::KeyCode::from_str(data.full_k.to_uppercase().as_str());
                        let res2=livesplit_hotkey::KeyCode::from_str(data.area_k.to_uppercase().as_str());
                        let k1;
                        let k2;

                        match res1 {
                            Ok(key)=>{k1=key;
                                if data.full_mod1.modifier==livesplit_hotkey::Modifiers::CONTROL || data.full_mod2.modifier==livesplit_hotkey::Modifiers::CONTROL{
                                    if data.full_mod1.modifier==livesplit_hotkey::Modifiers::ALT || data.full_mod2.modifier==livesplit_hotkey::Modifiers::ALT{
                                        if k1 == livesplit_hotkey::KeyCode::from_str("S").unwrap(){
                                            data.err = true;
                                            return;
                                        }
                                    }
                                }
                                if data.full_mod1.modifier==livesplit_hotkey::Modifiers::CONTROL && data.full_mod2.modifier==livesplit_hotkey::Modifiers::empty() &&
                                (k1 == livesplit_hotkey::KeyCode::from_str("C").unwrap() || k1 == livesplit_hotkey::KeyCode::from_str("S").unwrap()){
                                    data.err = true;
                                    return;
                                } 
                                data.err=false}
                            Err(_err)=>{data.err=true;return;}
                        }

                        match res2 {
                            Ok(key)=>{k2=key;
                                if data.area_mod1.modifier==livesplit_hotkey::Modifiers::CONTROL || data.area_mod2.modifier==livesplit_hotkey::Modifiers::CONTROL{
                                    if data.area_mod1.modifier==livesplit_hotkey::Modifiers::ALT || data.area_mod2.modifier==livesplit_hotkey::Modifiers::ALT{
                                        if k2 == livesplit_hotkey::KeyCode::from_str("S").unwrap(){
                                            data.err = true;
                                            return;
                                        }
                                    }
                                }
                                if data.area_mod1.modifier==livesplit_hotkey::Modifiers::CONTROL && data.area_mod2.modifier==livesplit_hotkey::Modifiers::empty() &&
                                (k2 == livesplit_hotkey::KeyCode::from_str("C").unwrap() || k2 == livesplit_hotkey::KeyCode::from_str("S").unwrap()){
                                    data.err = true;
                                    return;
                                } 
                                data.err=false}
                            Err(_err)=>{data.err=true;return;}
                        }

                        if data.full_mod1.modifier==livesplit_hotkey::Modifiers::empty(){
                            if data.full_mod2.modifier==livesplit_hotkey::Modifiers::empty(){
                                data.full_mod1=data.full_mod3.clone();
                            }else {
                                data.full_mod1=data.full_mod2.clone();
                            }
                        }else {
                            if data.full_mod2.modifier==livesplit_hotkey::Modifiers::empty(){
                                data.full_mod2=data.full_mod3.clone();
                            }
                        }


                        if data.full_mod1==data.full_mod2{
                            data.full_mod2=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }
                        if data.full_mod1==data.full_mod3{
                            data.full_mod3=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }
                        if data.full_mod2==data.full_mod3{
                            data.full_mod3=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }

                        if data.area_mod1.modifier==livesplit_hotkey::Modifiers::empty(){
                            if data.area_mod2.modifier==livesplit_hotkey::Modifiers::empty(){
                                data.area_mod1=data.area_mod3.clone();
                            }else {
                                data.area_mod1=data.area_mod2.clone();
                            }
                        }else {
                            if data.area_mod2.modifier==livesplit_hotkey::Modifiers::empty(){
                                data.area_mod2=data.area_mod3.clone();
                            }
                        }

                        if data.area_mod1==data.area_mod2{
                            data.area_mod2=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }
                        if data.area_mod1==data.area_mod3{
                            data.area_mod3=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }
                        if data.area_mod2==data.area_mod3{
                            data.area_mod3=MyModifier{modifier:livesplit_hotkey::Modifiers::empty()};
                        }

                        if data.full_mod1.modifier!=data.full_mods.0 ||data.full_mod2.modifier!=data.full_mods.1 ||
                        data.full_mod3.modifier!=data.full_mods.2 ||data.full_key!=k1 {
                            data.full_mods.0=data.full_mod1.modifier;
                            data.full_mods.1=data.full_mod2.modifier;
                            data.full_mods.2=data.full_mod3.modifier;
                            data.full_key=k1;

                            data.sender.send((livesplit_hotkey::Hotkey{key_code:data.full_key,modifiers:data.full_mods.0|data.full_mods.1|data.full_mods.2},1)).expect("Error shortcut");

                        }

                        if data.area_mod1.modifier!=data.area_mods.0 ||data.area_mod2.modifier!=data.area_mods.1 ||
                        data.area_mod3.modifier!=data.area_mods.2 ||data.area_key!=k2 {
                            data.area_mods.0=data.area_mod1.modifier;
                            data.area_mods.1=data.area_mod2.modifier;
                            data.area_mods.2=data.area_mod3.modifier;
                            data.area_key=k2;

                            data.sender.send((livesplit_hotkey::Hotkey{key_code:data.area_key,modifiers:data.area_mods.0|data.area_mods.1|data.area_mods.2},2)).expect("Error shortcut");

                        }

                        ctx.window().close();

                        //data.sender.send()
                    })
                )
                .with_child(
                    Button::new("Annulla")
                    .on_click(|ctx,data: &mut AppState, _env|{

                        data.full_mod1.modifier=data.full_mods.0;
                        data.full_mod2.modifier=data.full_mods.1;
                        data.full_mod3.modifier=data.full_mods.2;
                        data.full_k=data.full_key.name().to_string().pop().unwrap().to_string();
                        data.area_mod1.modifier=data.area_mods.0;
                        data.area_mod2.modifier=data.area_mods.1;
                        data.area_mod3.modifier=data.area_mods.2;
                        data.area_k=data.area_key.name().to_string().pop().unwrap().to_string();

                        ctx.window().close();
                        
                    })
                )
            ))
            .align_vertical(UnitPoint::LEFT)
        ).controller(ShortcutController{})
}

pub fn drag_motion_ui(is_full: bool) -> impl Widget<AppState> {
    let paint = Painter::new(move |ctx, data: &AppState, _env| {
        if !is_full {
            if let (Some(start), Some(end)) = (data.rect.start_point, data.rect.end_point) {
                let rect = druid::Rect::from_points(start, end);
                ctx.fill(
                    rect,
                    &Color::rgba(0.0, 0.0, 0.0, data.selection_transparency),
                );
                //ctx.stroke(rect, &druid::Color::WHITE, 1.0);
            }
        }
    })
    .controller(AreaController {
        id_t: TimerToken::next(),
        id_t2: TimerToken::next(),
        flag: is_full,
        display: None,
    })
    .center();

    Flex::column().with_child(paint)
}

pub fn show_edit() -> impl Widget<AppState>{

    let font: Font<'_> = Font::try_from_vec(Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8])).unwrap();

    let crop = ImageBuf::from_file("./icon/crop.png").unwrap();
    let marker = ImageBuf::from_file("./icon/marker.png").unwrap();
    let ellipse = ImageBuf::from_file("./icon/ellipse.png").unwrap();
    let square = ImageBuf::from_file("./icon/square.png").unwrap();
    let arrow = ImageBuf::from_file("./icon/right-arrow.png").unwrap();
    let text = ImageBuf::from_file("./icon/text.png").unwrap();
    let pencil = ImageBuf::from_file("./icon/pencil.png").unwrap();
    let color_wheel = ImageBuf::from_file("./icon/color-wheel.png").unwrap();
    

    Either::new(
        |data, _env| data.tool_window.tool == Tools::Resize,
        Flex::row()
                .with_child(Button::new("salva").on_click(
                    move |_ctx, data: &mut AppState, _env| {
                                let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
                                    ImageBuffer::from_vec(
                                        data.tool_window.img.width() as u32,
                                        data.tool_window.img.height() as u32,
                                        data.tool_window
                                            .img
                                            .clone()
                                            .raw_pixels()
                                            .to_vec(),
                                    )
                                    .unwrap();
                                let im = image.sub_image(
                                    ((data.rect.start_point.unwrap().x
                                        - data.tool_window.origin.x)
                                        * (data.tool_window.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        as u32,
                                    ((data.rect.start_point.unwrap().y
                                        - data.tool_window.origin.y)
                                        * (data.tool_window.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        as u32,
                                    (data.rect.size.width * data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width)
                                        as u32,
                                    (data.rect.size.height * data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height)
                                        as u32,
                                );
                                let imm = im.to_image();

                                data.tool_window.img = ImageBuf::from_raw(
                                    imm.clone().into_raw(),
                                    druid::piet::ImageFormat::RgbaPremul,
                                    (data.rect.size.width * data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width)
                                        as usize,
                                    (data.rect.size.height * data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height)
                                        as usize,
                                );

                                let width = data.tool_window.img.width() as f64;
                                let height = data.tool_window.img.height() as f64;

                                data.tool_window.img_size.width = data.tool_window.width;
                                data.tool_window.img_size.height =
                                    height / (width / data.tool_window.width);
                                if data.tool_window.img_size.height > data.tool_window.height {
                                    data.tool_window.img_size.height = data.tool_window.height;
                                    data.tool_window.img_size.width =
                                        width / (height / data.tool_window.height);
                                }

                                data.tool_window.origin = druid::Point::new(
                                    data.tool_window.center.x
                                        - (data.tool_window.img_size.width / 2.),
                                    data.tool_window.center.y
                                        - (data.tool_window.img_size.height / 2.),
                                );

                                data.tool_window.tool=Tools::No;
                                data.tool_window.draws.push((Draw::Resize{res:data.rect.clone()},Tools::Resize,data.color.clone()));
                            }
                ))
                .with_child(Button::new("annulla")
                        .on_click(|_ctx, data: &mut AppState, _env| {
                            
                            data.tool_window.rect_stroke = 0.;
                            data.tool_window.rect_transparency = 0.;
                                
                            data.tool_window.tool = Tools::No;
                        })
                        .padding(5.),
                ),
        Flex::row()
        .with_child(
            Image::new(crop)
                .fix_size(20., 20.)
                .on_click(|_ctx: &mut EventCtx, data: &mut AppState, _env| {
                    data.tool_window.tool = Tools::Resize;
                    data.resize = true;
                    data.annulla = false;

                    let rect = druid::Rect::from_center_size(
                        data.tool_window.center,
                        data.tool_window.img_size,
                    );
                    data.rect.size = data.tool_window.img_size;
                    data.rect.start_point.replace(rect.origin());
                    data.rect
                        .end_point
                        .replace(Point::new(rect.max_x(), rect.max_y()));
                    data.rect.p2 = Some(Point::new(rect.max_x(), 0.));
                    data.rect.p3 = Some(Point::new(0., rect.max_y()));
                    data.tool_window.rect_stroke = 2.0;
                    data.tool_window.rect_transparency = 0.4;
                })
                .border(Color::WHITE, 1.)
                .background(Color::rgb8(0, 0, 0))
                .padding(5.)
                .tooltip("ritaglia"),
            )
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Highlight,
                Image::new(marker.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Highlight
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 3.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("evidenziatore"),
                    
                    Image::new(marker.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Highlight
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 1.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("evidenziatore")
            ))
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Ellipse,
                Image::new(ellipse.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Ellipse;
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 3.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("ellisse"),
                    Image::new(ellipse.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Ellipse;
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 1.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("ellisse")
            ))
            .with_child(Either::new(
                |data: &AppState, _env| data.tool_window.tool == Tools::Ellipse,
                Flex::row()
                .with_child(
                Either::new(
                    |data: &AppState, _env| data.fill_shape,
                    Label::new("Pieno"),
                    Label::new("Vuoto"),
                ))
                .with_child(Switch::new().lens(AppState::fill_shape)),
                Label::new(""),
            ))
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Rectangle,
                Image::new(square.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Rectangle;
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 3.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("rettangolo"),

                    Image::new(square.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Rectangle;
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 1.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("rettangolo")
            ))
            .with_child(Either::new(
                |data: &AppState, _env| data.tool_window.tool == Tools::Rectangle,
                Flex::row()
                .with_child(
                Either::new(
                    |data: &AppState, _env| data.fill_shape,
                    Label::new("Pieno"),
                    Label::new("Vuoto"),
                ))
                .with_child(Switch::new().lens(AppState::fill_shape)),
                Label::new(""),
            ))
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Arrow,
                Image::new(arrow.clone())
                .fix_size(20., 20.)
                .on_click(|_ctx, data: &mut AppState, _: &Env| {
                    data.tool_window.tool = Tools::Arrow
                    //data.line_thickness = 10.;s
                })
                .border(Color::WHITE, 3.)
                .background(Color::rgb8(0, 0, 0))
                .padding(5.)
                .tooltip("freccia"),

                Image::new(arrow.clone())
                .fix_size(20., 20.)
                .on_click(|_ctx, data: &mut AppState, _: &Env| {
                    data.tool_window.tool = Tools::Arrow
                    //data.line_thickness = 10.;s
                })
                .border(Color::WHITE, 1.)
                .background(Color::rgb8(0, 0, 0))
                .padding(5.)
                .tooltip("freccia"),
            ))
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Text,
                Image::new(text.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Text
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 3.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("testo"),
                    
                    Image::new(text.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.tool_window.tool = Tools::Text
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 1.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("testo")
            ))
            .with_child(
                Container::new(
                Either::new(
                    |data:&AppState,_env| data.tool_window.tool==Tools::Text,
                    Flex::column()
                    .with_child(Container::new(
                        TextBox::multiline()
                            .with_placeholder("scrivi qui")
                            .fix_width(300.)
                            .fix_height(50.)
                            .lens(AppState::text)
                    ))
                    .with_child(
                        Button::new("salva")
                        .on_click(move |_ctx, data: &mut AppState, _: &Env| {
                            if let Some(point) = data.tool_window.text_pos {
                                let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
                                    ImageBuffer::from_vec(
                                        data.tool_window.img.width() as u32,
                                        data.tool_window.img.height() as u32,
                                        data.tool_window.img.clone().raw_pixels().to_vec(),
                                    )
                                    .unwrap();
    
                                let color = data.color.as_rgba8();
    
                                let strings = data.text.lines();
    
                                let mut deref = -16.;
                                for s in strings {
                                    imageproc::drawing::draw_text_mut(
                                        &mut image,
                                        Rgba([color.0, color.1, color.2, 255]),
                                        ((point.x - data.tool_window.origin.x)* (data.tool_window.img.width() as f64/ data.tool_window.img_size.width))as i32,
                                        ((point.y + deref - data.tool_window.origin.y)
                                            * (data.tool_window.img.height() as f64
                                                / data.tool_window.img_size.height))
                                            as i32,
                                        rusttype::Scale {
                                            x: 21.
                                                * (data.tool_window.img.width() as f64
                                                    / data.tool_window.img_size.width)
                                                    as f32,
                                            y: 22.
                                                * (data.tool_window.img.height() as f64
                                                    / data.tool_window.img_size.height)
                                                    as f32,
                                        },
                                        &font.clone(),
                                        s,
                                    );
    
                                    deref = deref + 25.;
                                }
    
                                data.tool_window.img = ImageBuf::from_raw(
                                    image.clone().into_raw(),
                                    druid::piet::ImageFormat::RgbaPremul,
                                    image.clone().width() as usize,
                                    image.clone().height() as usize,
                                );
                            }
    
                            data.tool_window.draws.push((Draw::Text{text:data.text.clone(),text_pos:data.tool_window.text_pos.unwrap(),font:font.clone()},Tools::Text,data.color.clone()));
                            data.tool_window.text_pos = None;
                            data.text = "".to_string();
                            //data.line_thickness = 10.;s
                        }
                        )
                        .disabled_if(|data,_env|{
                            data.text==""||data.tool_window.text_pos==None
                        })
                    ),
                        Label::new("")
                )).boxed()
            )
            .with_child(Either::new(|data, _env| data.tool_window.tool == Tools::Pencil,
                Image::new(pencil.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.color = data.color.with_alpha(0.);
                        data.tool_window.tool = Tools::Pencil
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 3.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("penna"),

                    Image::new(pencil.clone())
                    .fix_size(20., 20.)
                    .on_click(|_ctx, data: &mut AppState, _: &Env| {
                        data.color = data.color.with_alpha(0.);
                        data.tool_window.tool = Tools::Pencil
                        //data.line_thickness = 10.;s
                    })
                    .border(Color::WHITE, 1.)
                    .background(Color::rgb8(0, 0, 0))
                    .padding(5.)
                    .tooltip("penna"),
            ))
            .with_child(
                Button::new("indietro")
                .on_click(|_ctx,data: &mut AppState,_env|{
                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                        data.img.width() as u32,
                        data.img.height() as u32,
                        data.img.clone().raw_pixels().to_vec(),
                    )
                    .unwrap();

                    data.tool_window.draws.pop().unwrap();

                    let width = data.img.width() as f64;
                            let height = data.img.height() as f64;

                            data.tool_window.img_size.width = data.tool_window.width;
                            data.tool_window.img_size.height = height / (width / data.tool_window.width);
                            if data.tool_window.img_size.height > data.tool_window.height {
                                data.tool_window.img_size.height = data.tool_window.height;
                                data.tool_window.img_size.width = width / (height / data.tool_window.height);
                            }

                            data.tool_window.origin = druid::Point::new(
                                data.tool_window.center.x - (data.tool_window.img_size.width / 2.),
                                data.tool_window.center.y - (data.tool_window.img_size.height / 2.),
                            );

                    for d in data.tool_window.draws.clone(){
                        let w=image.width();
                        let h=image.height();
                        let color=d.2.as_rgba8();
                        match d.0{
                            Draw::Resize{res}=>{

                                data.tool_window.img_size.width = data.tool_window.width;
                                data.tool_window.img_size.height =
                                    h as f64/ (w as f64 / data.tool_window.width);
                                if data.tool_window.img_size.height > data.tool_window.height {
                                    data.tool_window.img_size.height = data.tool_window.height;
                                    data.tool_window.img_size.width =
                                        w as f64/ (h as f64 / data.tool_window.height);
                                }

                                data.tool_window.origin = druid::Point::new(
                                    data.tool_window.center.x
                                        - (data.tool_window.img_size.width / 2.),
                                    data.tool_window.center.y
                                        - (data.tool_window.img_size.height / 2.),
                                );


                                let im = image.sub_image(
                                    ((res.start_point.unwrap().x
                                        - data.tool_window.origin.x)
                                        * (image.width() as f64
                                            / data.tool_window.img_size.width))
                                        as u32,
                                    ((res.start_point.unwrap().y
                                        - data.tool_window.origin.y)
                                        * (image.height() as f64
                                            / data.tool_window.img_size.height))
                                        as u32,
                                    (res.size.width * image.width() as f64
                                        / data.tool_window.img_size.width)
                                        as u32,
                                    (res.size.height * image.height() as f64
                                        / data.tool_window.img_size.height)
                                        as u32,
                                );
                                let imm = im.to_image();

                                image=imm;

                                let width = image.width() as f64;
                                let height = image.height() as f64;

                                data.tool_window.img_size.width = data.tool_window.width;
                                data.tool_window.img_size.height =
                                    height / (width / data.tool_window.width);
                                if data.tool_window.img_size.height > data.tool_window.height {
                                    data.tool_window.img_size.height = data.tool_window.height;
                                    data.tool_window.img_size.width =
                                        width / (height / data.tool_window.height);
                                }

                                data.tool_window.origin = druid::Point::new(
                                    data.tool_window.center.x
                                        - (data.tool_window.img_size.width / 2.),
                                    data.tool_window.center.y
                                        - (data.tool_window.img_size.height / 2.),
                                );
                            }
                            Draw::Free { points }=>{

                                if !points.is_empty()&&points.len()>=1 {
                                    
                                    for i in 0..points.len() - 1 {
                                        let mut line = FreeRect::new(
                                            points[i],
                                            points[i],
                                            points[i + 1],
                                            points[i + 1],
                                        );

                                        let dx = ((line.p3.x - line.p1.x) as f64).abs();
                                        let sx;
                                        if line.p1.x < line.p3.x {
                                            sx = 1;
                                        } else {
                                            sx = -1;
                                        }
                                        let dy = -((line.p3.y - line.p1.y) as f64).abs();
                                        let sy;
                                        if line.p1.y < line.p3.y {
                                            sy = 1;
                                        } else {
                                            sy = -1;
                                        }
                                        let mut err = dx + dy;
                                        let mut e2;

                                        for _i in 0..=1 {
                                            e2 = err * 2.;
                                            if e2 >= dy {
                                                err = err + dy;
                                                line.p1.y = line.p1.y + sy;
                                                line.p2.y = line.p2.y - sy;
                                                line.p3.y = line.p3.y - sy;
                                                line.p4.y = line.p4.y + sy;
                                            }
                                            if e2 <= dx {
                                                err = err + dx;
                                                line.p1.x = line.p1.x - sx;
                                                line.p2.x = line.p2.x + sx;
                                                line.p3.x = line.p3.x + sx;
                                                line.p4.x = line.p4.x - sx;
                                            }
                                        }

                                        line.p1.x = ((line.p1.x as f64 - data.tool_window.origin.x)
                                            * (image.width() as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p1.y = ((line.p1.y as f64 - data.tool_window.origin.y)
                                            * (image.height() as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p2.x = ((line.p2.x as f64 - data.tool_window.origin.x)
                                            * (image.width() as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p2.y = ((line.p2.y as f64 - data.tool_window.origin.y)
                                            * (image.height() as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p3.x = ((line.p3.x as f64 - data.tool_window.origin.x)
                                            * (image.width() as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p3.y = ((line.p3.y as f64 - data.tool_window.origin.y)
                                            * (image.height() as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p4.x = ((line.p4.x as f64 - data.tool_window.origin.x)
                                            * (image.width() as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p4.y = ((line.p4.y as f64 - data.tool_window.origin.y)
                                            * (image.height() as f64 / data.tool_window.img_size.height))
                                            as i32;

                                        if line.p1 != line.p4 {
                                            imageproc::drawing::draw_polygon_mut(
                                                &mut image,
                                                &[line.p1, line.p2, line.p3, line.p4],
                                                Rgba([color.0, color.1, color.2, color.3]),
                                            );
                                        }

                                        imageproc::drawing::draw_filled_circle_mut(
                                            &mut image,
                                            (
                                                ((points[i].x - data.tool_window.origin.x)
                                                    * (w as f64 / data.tool_window.img_size.width))
                                                    as i32,
                                                ((points[i].y - data.tool_window.origin.y)
                                                    * (h as f64 / data.tool_window.img_size.height))
                                                    as i32,
                                            ),
                                            2 * (h as f64 / data.tool_window.img_size.height)
                                                as i32,
                                            Rgba([color.0, color.1, color.2, color.3]),
                                        );                        }

                                    imageproc::drawing::draw_filled_circle_mut(
                                        &mut image,
                                        (
                                            ((points.last().unwrap().x - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32,
                                            ((points.last().unwrap().y - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32,
                                        ),
                                        3 * (h as f64 / data.tool_window.img_size.height)
                                            as i32,
                                        Rgba([color.0, color.1, color.2, color.3]),
                                    );

                                }else
                                {
                                    imageproc::drawing::draw_filled_circle_mut(
                                        &mut image,
                                        (
                                            ((points[0].x - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32,
                                            ((points[0].y - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32,
                                        ),
                                        3 * (h as f64 / data.tool_window.img_size.height)
                                            as i32,
                                        Rgba([color.0, color.1, color.2, color.3]),
                                    );
                                }

                            }
                            Draw::Shape { shape }=>{
                                match d.1 {
                                    Tools::Ellipse=>{
                                        if shape.filled{
                                            imageproc::drawing::draw_filled_ellipse_mut(
                                                &mut image,
                                                (
                                                    ((shape.center.unwrap().x
                                                        - data.tool_window.origin.x)
                                                        * (w as f64 / data.tool_window.img_size.width))
                                                        as i32,
                                                    ((shape.center.unwrap().y
                                                        - data.tool_window.origin.y)
                                                        * (h as f64 / data.tool_window.img_size.height))
                                                        as i32,
                                                ),
                                                (shape.radii.unwrap().x
                                                    * (w as f64 / data.tool_window.img_size.width))
                                                    as i32,
                                                (shape.radii.unwrap().y
                                                    * (h as f64 / data.tool_window.img_size.height))
                                                    as i32,
                                                Rgba([color.0, color.1, color.2, 255]),
                                            );
                                        }else {
                                            for i in -50..50 {
                                                imageproc::drawing::draw_hollow_ellipse_mut(
                                                    &mut image,
                                                    (
                                                        ((shape.center.unwrap().x
                                                            - data.tool_window.origin.x)
                                                            * (w as f64
                                                                / data.tool_window.img_size.width))
                                                            .round() as i32,
                                                        ((shape.center.unwrap().y
                                                            - data.tool_window.origin.y)
                                                            * (h as f64
                                                                / data.tool_window.img_size.height))
                                                            .round() as i32,
                                                    ),
                                                    ((shape.radii.unwrap().x - i as f64 / 20.)
                                                        * (w as f64 / data.tool_window.img_size.width))
                                                        as i32,
                                                    ((shape.radii.unwrap().y - i as f64 / 20.)
                                                        * (h as f64 / data.tool_window.img_size.height))
                                                        as i32,
                                                    Rgba([color.0, color.1, color.2, 255]),
                                                );
                                            }
                                        }
                                    }
                                    Tools::Rectangle=>{
                                        if shape.filled{
                                            imageproc::drawing::draw_filled_rect_mut(
                                                &mut image,
                                                imageproc::rect::Rect::at(
                                                    ((shape
                                                        .start_point
                                                        .unwrap()
                                                        .x
                                                        .min(shape.end_point.unwrap().x)
                                                        - data.tool_window.origin.x)
                                                        * (w as f64 / data.tool_window.img_size.width))
                                                        as i32,
                                                    ((shape
                                                        .start_point
                                                        .unwrap()
                                                        .y
                                                        .min(shape.end_point.unwrap().y)
                                                        - data.tool_window.origin.y)
                                                        * (h as f64 / data.tool_window.img_size.height))
                                                        as i32,
                                                )
                                                .of_size(
                                                    (((shape.start_point.unwrap().x
                                                        - shape.end_point.unwrap().x)
                                                        .abs())
                                                        * (w as f64 / data.tool_window.img_size.width))
                                                        as u32,
                                                    (((shape.start_point.unwrap().y
                                                        - shape.end_point.unwrap().y)
                                                        .abs())
                                                        * (h as f64 / data.tool_window.img_size.height))
                                                        as u32,
                                                ),
                                                Rgba([color.0, color.1, color.2, 255]),
                                            );
                                        }else {
                                            for i in -20..20 {
                                                imageproc::drawing::draw_hollow_rect_mut(
                                                    &mut image,
                                                    imageproc::rect::Rect::at(
                                                        ((shape
                                                            .start_point
                                                            .unwrap()
                                                            .x
                                                            .min(shape.end_point.unwrap().x)
                                                            + i as f64 / 10.
                                                            - data.tool_window.origin.x)
                                                            * (w as f64
                                                                / data.tool_window.img_size.width))
                                                            as i32,
                                                        ((shape
                                                            .start_point
                                                            .unwrap()
                                                            .y
                                                            .min(shape.end_point.unwrap().y)
                                                            + i as f64 / 10.
                                                            - data.tool_window.origin.y)
                                                            * (h as f64
                                                                / data.tool_window.img_size.height))
                                                            as i32,
                                                    )
                                                    .of_size(
                                                        ((((shape.start_point.unwrap().x
                                                            - shape.end_point.unwrap().x)
                                                            .abs()
                                                            - 2. * i as f64 / 10.)
                                                            * (w as f64
                                                                / data.tool_window.img_size.width))
                                                            as u32)
                                                            .max(1),
                                                        ((((shape.start_point.unwrap().y
                                                            - shape.end_point.unwrap().y)
                                                            .abs()
                                                            - 2. * i as f64 / 10.)
                                                            * (h as f64
                                                                / data.tool_window.img_size.height))
                                                            as u32)
                                                            .max(1),
                                                    ),
                                                    Rgba([color.0, color.1, color.2, 255]),
                                                );
                                            }
                                        }
                                    }
                                    Tools::Highlight=>{
                                        let mut top =ImageBuffer::new(data.tool_window.img.width() as u32, data.tool_window.img.height() as u32);

                                        let mut line = FreeRect::new(
                                            shape.start_point.unwrap(),
                                            shape.start_point.unwrap(),
                                            shape.end_point.unwrap(),
                                            shape.end_point.unwrap(),
                                        );

                                        let dx = ((line.p3.x - line.p1.x) as f64).abs();
                                        let sx;
                                        if line.p1.x < line.p3.x {
                                            sx = 1;
                                        } else {
                                            sx = -1;
                                        }
                                        let dy = -((line.p3.y - line.p1.y) as f64).abs();
                                        let sy;
                                        if line.p1.y < line.p3.y {
                                            sy = 1;
                                        } else {
                                            sy = -1;
                                        }
                                        let mut err = dx + dy;
                                        let mut e2;

                                        for _i in 0..=4 {
                                            e2 = err * 2.;
                                            if e2 >= dy {
                                                err = err + dy;
                                                line.p1.y = line.p1.y + sy;
                                                line.p2.y = line.p2.y - sy;
                                                line.p3.y = line.p3.y - sy;
                                                line.p4.y = line.p4.y + sy;
                                            }
                                            if e2 <= dx {
                                                err = err + dx;
                                                line.p1.x = line.p1.x - sx;
                                                line.p2.x = line.p2.x + sx;
                                                line.p3.x = line.p3.x + sx;
                                                line.p4.x = line.p4.x - sx;
                                            }
                                        }

                                        line.p1.x = ((line.p1.x as f64 - data.tool_window.origin.x)
                                            * (w as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p1.y = ((line.p1.y as f64 - data.tool_window.origin.y)
                                            * (h as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p2.x = ((line.p2.x as f64 - data.tool_window.origin.x)
                                            * (w as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p2.y = ((line.p2.y as f64 - data.tool_window.origin.y)
                                            * (h as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p3.x = ((line.p3.x as f64 - data.tool_window.origin.x)
                                            * (w as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p3.y = ((line.p3.y as f64 - data.tool_window.origin.y)
                                            * (h as f64 / data.tool_window.img_size.height))
                                            as i32;
                                        line.p4.x = ((line.p4.x as f64 - data.tool_window.origin.x)
                                            * (w as f64 / data.tool_window.img_size.width))
                                            as i32;
                                        line.p4.y = ((line.p4.y as f64 - data.tool_window.origin.y)
                                            * (h as f64 / data.tool_window.img_size.height))
                                            as i32;

                                        imageproc::drawing::draw_polygon_mut(
                                            &mut top,
                                            &[line.p1, line.p2, line.p3, line.p4],
                                            Rgba([color.0, color.1, color.2, color.3]),
                                        );

                                        image::imageops::overlay(&mut image, &top, 0, 0);
                                    }
                                    Tools::Arrow=>{
                                        
                                        let end = shape.end_point.unwrap();
                                        let start = shape.start_point.unwrap();

                                        if end == start {
                                        } else {
                                            let cos = 0.866;
                                            let sin = 0.500;
                                            let dx = end.x - start.x;
                                            let dy = end.y - start.y;
                                            let end1 = druid::Point::new(
                                                end.x - (dx * cos + dy * -sin) * 2. / 5.,
                                                end.y - (dx * sin + dy * cos) * 2. / 5.,
                                            );
                                            let end2 = druid::Point::new(
                                                end.x - (dx * cos + dy * sin) * 2. / 5.,
                                                end.y - (dx * -sin + dy * cos) * 2. / 5.,
                                            );

                                            let mut body = FreeRect::new(
                                                shape.start_point.unwrap(),
                                                shape.start_point.unwrap(),
                                                shape.end_point.unwrap(),
                                                shape.end_point.unwrap(),
                                            );
                                            let mut line1 = FreeRect::new(
                                                end1,
                                                end1,
                                                shape.end_point.unwrap(),
                                                shape.end_point.unwrap(),
                                            );
                                            let mut line2 = FreeRect::new(
                                                end2,
                                                end2,
                                                shape.end_point.unwrap(),
                                                shape.end_point.unwrap(),
                                            );

                                            let dx = ((body.p3.x - body.p1.x) as f64).abs();
                                            let sx;
                                            if body.p1.x < body.p3.x {
                                                sx = 1;
                                            } else {
                                                sx = -1;
                                            }
                                            let dy = -((body.p3.y - body.p1.y) as f64).abs();
                                            let sy;
                                            if body.p1.y < body.p3.y {
                                                sy = 1;
                                            } else {
                                                sy = -1;
                                            }
                                            let mut err = dx + dy;
                                            let mut e2;

                                            for _i in 0..=2 {
                                                e2 = err * 2.;
                                                if e2 >= dy {
                                                    err = err + dy;
                                                    body.p1.y = body.p1.y + sy;
                                                    body.p2.y = body.p2.y - sy;
                                                    body.p3.y = body.p3.y - sy;
                                                    body.p4.y = body.p4.y + sy;

                                                    line1.p1.y = line1.p1.y + sy;
                                                    line1.p2.y = line1.p2.y - sy;
                                                    line1.p3.y = line1.p3.y - sy;
                                                    line1.p4.y = line1.p4.y + sy;

                                                    line2.p1.y = line2.p1.y + sy;
                                                    line2.p2.y = line2.p2.y - sy;
                                                    line2.p3.y = line2.p3.y - sy;
                                                    line2.p4.y = line2.p4.y + sy;
                                                }
                                                if e2 <= dx {
                                                    err = err + dx;
                                                    body.p1.x = body.p1.x - sx;
                                                    body.p2.x = body.p2.x + sx;
                                                    body.p3.x = body.p3.x + sx;
                                                    body.p4.x = body.p4.x - sx;

                                                    line1.p1.x = line1.p1.x - sx;
                                                    line1.p2.x = line1.p2.x + sx;
                                                    line1.p3.x = line1.p3.x + sx;
                                                    line1.p4.x = line1.p4.x - sx;

                                                    line2.p1.x = line2.p1.x - sx;
                                                    line2.p2.x = line2.p2.x + sx;
                                                    line2.p3.x = line2.p3.x + sx;
                                                    line2.p4.x = line2.p4.x - sx;
                                                }
                                            }

                                            body.p1.x = ((body.p1.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            body.p1.y = ((body.p1.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            body.p2.x = ((body.p2.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            body.p2.y = ((body.p2.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            body.p3.x = ((body.p3.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            body.p3.y = ((body.p3.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            body.p4.x = ((body.p4.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            body.p4.y = ((body.p4.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;

                                            line1.p1.x = ((line1.p1.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line1.p1.y = ((line1.p1.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line1.p2.x = ((line1.p2.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line1.p2.y = ((line1.p2.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line1.p3.x = ((line1.p3.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line1.p3.y = ((line1.p3.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line1.p4.x = ((line1.p4.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line1.p4.y = ((line1.p4.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;

                                            line2.p1.x = ((line2.p1.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line2.p1.y = ((line2.p1.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line2.p2.x = ((line2.p2.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line2.p2.y = ((line2.p2.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line2.p3.x = ((line2.p3.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line2.p3.y = ((line2.p3.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;
                                            line2.p4.x = ((line2.p4.x as f64 - data.tool_window.origin.x)
                                                * (w as f64 / data.tool_window.img_size.width))
                                                as i32;
                                            line2.p4.y = ((line2.p4.y as f64 - data.tool_window.origin.y)
                                                * (h as f64 / data.tool_window.img_size.height))
                                                as i32;

                                            imageproc::drawing::draw_polygon_mut(
                                                &mut image,
                                                &[body.p1, body.p2, body.p3, body.p4],
                                                Rgba([color.0, color.1, color.2, color.3]),
                                            );

                                            imageproc::drawing::draw_polygon_mut(
                                                &mut image,
                                                &[line1.p1, line1.p2, line1.p3, line1.p4],
                                                Rgba([color.0, color.1, color.2, color.3]),
                                            );
                                            imageproc::drawing::draw_polygon_mut(
                                                &mut image,
                                                &[line2.p1, line2.p2, line2.p3, line2.p4],
                                                Rgba([color.0, color.1, color.2, color.3]),
                                            );

                                        }            
                                    }
                                    _=>{}
                                }
                            }
                            Draw::Text { text, text_pos ,font}=>{
                                let strings = text.lines();
    
                                let mut deref = -16.;
                                for s in strings {
                                    imageproc::drawing::draw_text_mut(
                                        &mut image,
                                        Rgba([color.0, color.1, color.2, 255]),
                                        ((text_pos.x - data.tool_window.origin.x)* (w as f64/ data.tool_window.img_size.width))as i32,
                                        ((text_pos.y + deref - data.tool_window.origin.y)
                                            * (h as f64
                                                / data.tool_window.img_size.height))
                                            as i32,
                                        rusttype::Scale {
                                            x: 21.
                                                * (w as f64
                                                    / data.tool_window.img_size.width)
                                                    as f32,
                                            y: 22.
                                                * (h as f64
                                                    / data.tool_window.img_size.height)
                                                    as f32,
                                        },
                                        &font.clone(),
                                        s,
                                    );
    
                                    deref = deref + 25.;
                                }
                                }
                        }
                    }

                    data.tool_window.img = ImageBuf::from_raw(
                        image.clone().into_raw(),
                        druid::piet::ImageFormat::RgbaPremul,
                        image.clone().width() as usize,
                        image.clone().height() as usize,
                    );                })
                .disabled_if(|data: &AppState,_env|{data.tool_window.draws.len()==0})
            )
            .with_child(ViewSwitcher::new(|data: &AppState, _env| data.color, |_ctx, data: &AppState, _env| match data.color.with_alpha(1.){
                Color::RED => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_red.png")).unwrap()
                    
                )
                .fix_size(30., 30.))}
                Color::BLACK => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_black.png")).unwrap(),
                )
                .fix_size(30., 30.))}
                Color::GREEN => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_green.png")).unwrap(),
                )
                .fix_size(30., 30.) )}
                Color::GRAY => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_grey.png")).unwrap(),
                )
                .fix_size(30., 30.) )}
                Color::WHITE => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_white.png")).unwrap(),
                )
                .fix_size(30., 30.))}
                Color::BLUE => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_blue.png")).unwrap(),
                )
                .fix_size(30., 30.))}
                Color::YELLOW => {Box::new(Image::new(
                    ImageBuf::from_data(include_bytes!("../icon/circle_yellow.png")).unwrap(),
                )
                .fix_size(30., 30.))}
                _ => {Box::new(Button::new(""))}
            } ))
            .with_child(Either::new(
                |data: &AppState, _env| !data.color_picker,
                Image::new(color_wheel)
                .fix_size(30., 30.)
                .on_click(|_ctx, data: &mut AppState, _: &Env| {
                    data.color_picker = true;
                }),
                Flex::row()
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_black.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::BLACK)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::BLACK;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_blue.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::BLUE)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::BLUE;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_green.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::GREEN)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::GREEN;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                         Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_grey.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::GRAY)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::GRAY;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_red.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::RED)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::RED;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_white.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::WHITE)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::WHITE;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    )
                    .with_child(
                        Image::new(
                            ImageBuf::from_data(include_bytes!("../icon/circle_yellow.png"))
                                .unwrap(),
                        )
                        .fix_size(20., 20.)
                        //Button::new("")
                        //.foreground(Color::YELLOW)
                        .on_click(|_ctx, data: &mut AppState, _: &Env| {
                            data.color = Color::YELLOW;
                            data.color=data.color.with_alpha(0.);
                            data.color_picker = false;
                        })
                        .padding(5.),
                    ))
            ),
            
        )
}

pub fn show_screen_ui() -> impl Widget<AppState> {
    let points = Vec::<Point>::new();
    let mut path = Vec::new();
    //let brush = BackgroundBrush::Color(druid::Color::rgb(255., 0., 0.));
    
    Flex::column()
        .with_child(show_edit())
        .with_child(
            Container::new(ZStack::new(
                SizedBox::new(
                    
                    Painter::new(|ctx, data: &AppState, _env| {
                        let image = ctx
                        .make_image(
                                data.tool_window.img.width(),
                                data.tool_window.img.height(),
                                data.tool_window.img.clone().raw_pixels(),
                                druid_shell::piet::ImageFormat::RgbaPremul,
                            )
                            .unwrap();

                        ctx.draw_image(
                            &image,
                            druid::Rect::from_center_size(
                                data.tool_window.center,
                                data.tool_window.img_size,
                            ),
                            druid_shell::piet::InterpolationMode::Bilinear,
                        );
                    }),
                )
                .width(800.)
                .height(500.)
                .background(BackgroundBrush::Color(druid::Color::rgb(0., 0., 0.))),
            )
            .with_centered_child(
                Painter::new(
                    move |ctx, data: &AppState, env|{ match data.tool_window.tool {
                        Tools::Resize => {
                            if let (Some(_start), Some(_end)) =
                                (data.rect.start_point, data.rect.end_point)
                            {
                                if data.rect.size.width <= data.tool_window.width
                                    && data.rect.size.height <= data.tool_window.height
                                {
                                    let shape = druid::Rect::from_points(
                                        data.rect.start_point.unwrap(),
                                        data.rect.end_point.unwrap(),
                                    );
                                    let start = data.rect.start_point.unwrap();
                                    let end = data.rect.end_point.unwrap();
                                    let diff_x = end.x - start.x;
                                    let diff_y = end.y - start.y;

                                    let grid1 = druid::Rect::from_points(
                                        Point::new(start.x + diff_x / 3.0, start.y),
                                        Point::new(start.x + diff_x / 3.0 * 2.0, end.y),
                                    );
                                    let grid2 = druid::Rect::from_points(
                                        Point::new(start.x, start.y + diff_y / 3.0),
                                        Point::new(end.x, start.y + diff_y / 3.0 * 2.0),
                                    );
                                    if data.tool_window.rect_stroke != 0.0 {
                                        ctx.stroke(grid1, &druid::Color::WHITE, 0.5);
                                        ctx.stroke(grid2, &druid::Color::WHITE, 0.5);
                                    }

                                    ctx.fill(
                                        shape,
                                        &Color::rgba(
                                            0.0,
                                            0.0,
                                            0.0,
                                            data.tool_window.rect_transparency,
                                        ),
                                    );
                                    ctx.stroke(
                                        shape,
                                        &druid::Color::WHITE,
                                        data.tool_window.rect_stroke,
                                    );
                                }
                            }
                        }
                        Tools::Ellipse => {
                            if let (Some(center), Some(_end)) = (
                                data.tool_window.shape.center,
                                data.tool_window.shape.end_point,
                            ) {
                                let color = data.color.as_rgba();
                                let shape = druid::kurbo::Ellipse::new(
                                    center,
                                    data.tool_window.shape.radii.unwrap(),
                                    0.0,
                                );

                                if !data.fill_shape {
                                    ctx.fill(shape, &Color::rgba(color.0, color.1, color.2, 0.));
                                    ctx.stroke(
                                        shape,
                                        &Color::rgba(color.0, color.1, color.2, 1.),
                                        5.,
                                    )
                                } else {
                                    ctx.fill(shape, &Color::rgba(color.0, color.1, color.2, 1.));
                                }
                            }
                        }
                        Tools::Rectangle=>{
                            if let (Some(start), Some(end)) = (data.tool_window.shape.start_point,data.tool_window.shape.end_point,) {
                                let color = data.color.as_rgba();
                                let shape = druid::kurbo::Rect::new(
                                    start.x,
                                    start.y,
                                    end.x,
                                    end.y,
                                );

                                if !data.fill_shape {
                                    ctx.fill(shape, &Color::rgba(color.0, color.1, color.2, 0.));
                                    ctx.stroke(
                                        shape,
                                        &Color::rgba(color.0, color.1, color.2, 1.),
                                        5.,
                                    )
                                } else {
                                    ctx.fill(shape, &Color::rgba(color.0, color.1, color.2, 1.));
                                }
                            }
                        }
                        Tools::Arrow => {
                            if let (Some(start), Some(end)) = (
                                data.tool_window.shape.start_point,
                                data.tool_window.shape.end_point,
                            ) {
                                let color = data.color.as_rgba();

                                let body = druid::kurbo::Line::new(start, end);
                                ctx.stroke(
                                    body,
                                    &Color::rgba(
                                        color.0,
                                        color.1,
                                        color.2,
                                        data.tool_window.rect_transparency,
                                    ),
                                    6.75,
                                );

                                let cos = 0.866;
                                let sin = 0.500;
                                let dx = end.x - start.x;
                                let dy = end.y - start.y;
                                let end1 = druid::Point::new(
                                    end.x - (dx * cos + dy * -sin) * 2. / 5.,
                                    end.y - (dx * sin + dy * cos) * 2. / 5.,
                                );
                                let end2 = druid::Point::new(
                                    end.x - (dx * cos + dy * sin) * 2. / 5.,
                                    end.y - (dx * -sin + dy * cos) * 2. / 5.,
                                );

                                ctx.stroke(
                                    druid::kurbo::Line::new(end, end1),
                                    &Color::rgba(
                                        color.0,
                                        color.1,
                                        color.2,
                                        data.tool_window.rect_transparency,
                                    ),
                                    6.75,
                                );

                                ctx.stroke(
                                    druid::kurbo::Line::new(end, end2),
                                    &Color::rgba(
                                        color.0,
                                        color.1,
                                        color.2,
                                        data.tool_window.rect_transparency,
                                    ),
                                    6.75,
                                );
                            }
                        }
                        Tools::Highlight => {
                            if let (Some(start), Some(end)) = (
                                data.tool_window.shape.start_point,
                                data.tool_window.shape.end_point,
                            ) {
                                let color = data.color.as_rgba();
                                let shape = druid::kurbo::Line::new(start, end);

                                ctx.stroke(
                                    shape,
                                    &Color::rgba(color.0, color.1, color.2, color.3),
                                    10.,
                                );
                            }
                        }
                        Tools::Pencil => {
                            if let Some(point) = data.tool_window.random_point {
                                let color = data.color.as_rgba();
                                let mut bez=druid::kurbo::BezPath::new();

                                path.push(point);

                                bez.move_to(path[0].clone());
                                for p in path.iter().skip(1){
                                    bez.line_to(p.clone());
                                }
                                
                                let circle = druid::kurbo::Circle::new(point, 5.);

                                ctx.fill(circle, &Color::rgba(color.0, color.1, color.2, color.3));
                                ctx.stroke(
                                    bez.clone(),
                                    &Color::rgba(color.0, color.1, color.2, color.3),
                                    5.,
                                );

                                if color.3 == 0.0 {
                                    path = Vec::new();
                                }
                                
                            }
                        }
                        Tools::Text => {
                            if let Some(point) = data.tool_window.text_pos {
                                let mut a = druid::text::TextLayout::new();

                                a.set_text_color(data.color);
                                a.set_font(druid::text::FontDescriptor {
                                    family: druid::text::FontFamily::SANS_SERIF,
                                    size: 20.,
                                    weight: druid::text::FontWeight::NORMAL,
                                    style: druid::text::FontStyle::Regular,
                                });

                                a.set_text(data.text.clone());

                                a.rebuild_if_needed(ctx.text(), env);

                                a.draw(ctx, druid::Point::new(point.x, point.y - 20.));
                            }
                        }
                        _ => {}
                    };
                }
                )
                .center()
            )
            .controller(AnnotationsController { points: points, flag: true }),
        ))
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
                .enabled_if(move |data: &AppState, _env| {
                    !data.img.size().is_empty()
                })
                .hotkey(SysMods::Cmd, "s"),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Salva come")) //posso scegliere il path
                .command(commands::SHOW_SAVE_PANEL.with(save_dialog))
                .enabled_if(move |data: &AppState, _env| !data.img.size().is_empty())
                .hotkey(SysMods::AltCmd, "s"),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Copia"))
                .on_activate(move |_ctx, data: &mut AppState, _env| {
                    //ctx.submit_command(commands::SHOW_SAVE_PANEL.with(save_dialog_options.clone()))
                    let img = ImageData {
                        width: data.tool_window.img.width(),
                        height: data.tool_window.img.height(),
                        bytes: Cow::from(data.tool_window.img.raw_pixels()),
                    };
                    let mut clip = Clipboard::new().unwrap();
                    clip.set_image(img).unwrap();
                })
                .enabled_if(move |data: &AppState, _env| {
                    !data.img.size().is_empty()
                })
                .hotkey(SysMods::Cmd, "c"),
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
