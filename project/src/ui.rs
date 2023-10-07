use crate::data::*;
use arboard::{Clipboard, ImageData};
use druid::widget::{
    BackgroundBrush, Button, Either, FillStrat, Flex, Image, Label, Painter, SizedBox, TextBox,
    ZStack,
};
use druid::{
    commands, Color, Env, EventCtx, FileDialogOptions, FileSpec, ImageBuf, LocalizedString, Menu,
    MenuItem, Point, RenderContext, Size, UnitPoint, Vec2, Widget, WidgetExt, WidgetPod,
    WindowDesc, WindowId, WindowLevel, WindowState,
};
use druid_shell::keyboard_types::Modifiers;
use druid_shell::TimerToken;
use druid_widget_nursery::DropdownSelect;
use image::{GenericImage, ImageBuffer, Rgba, SubImage};
use imageproc::filter;
use rusttype::Font;
use std::borrow::Cow;

pub fn build_ui(scale: f32, img: ImageBuf) -> impl Widget<AppState> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");
    //let scale = display_info[0].scale_factor;
    let mut width = (display_info[0].width as f32 * display_info[0].scale_factor) as u32;
    let mut height = (display_info[0].height as f32 * display_info[0].scale_factor) as u32;
    let mut pos = Point::new(0., 0.);
    
    for display in display_info.iter() {
        if display.x < 0 {
            if display.x + display.width as i32 == 0 {
                width += (display.width as f32 * display.scale_factor) as u32;
            } else {
                width = (width as i32 - display.x) as u32
            }
            pos.x = ((display.x as f32 / scale ) * display.scale_factor as f32) as f64;
        } else if display.x as f32/ scale  >= display_info[0].width as f32 {
            width += (display.width as f32 * display.scale_factor) as u32;
        }else{
            if (display.x as f32/ scale) + (display.width as f32 / scale) 
                > display_info[0].width as f32
            {
                width += (display.width as f32 * display.scale_factor) as u32 - (display_info[0].width as f32 * scale  - display.x as f32) as u32;
            }
        }

        if display.y < 0 { 
            if display.y + display.height as i32 == 0 {
                height += (display.height as f32 * display.scale_factor) as u32;
            } else {
                height = (height as i32 - display.y) as u32
            }
            pos.y = ((display.y as f32 / scale) * display.scale_factor as f32) as f64;
        } else if display.y as f32/ scale >= display_info[0].height as f32{
            height += (display.height as f32 * display.scale_factor) as u32;
        } else {
            if (display.y as f32/ scale ) + (display.height as f32 / scale)
                > display_info[0].height as f32
            {
                height += (display.height as f32 * display.scale_factor) as u32 - (display_info[0].height as f32 * scale - display.y as f32) as u32;
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
                            data.rect.p2 = Some(Point::new(0., data.size.y));
                            data.rect.p3 = Some(Point::new(data.size.x, 0.));
                            let new_win = WindowDesc::new(drag_motion_ui(true))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((width as f64, height as f64))
                                .resizable(false)
                                .set_position(pos);
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
                            println!("wid: {}, hei: {}, pos: {}", width, height, pos);
                            let new_win = WindowDesc::new(drag_motion_ui(false))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((width as f64, height as f64))
                                .resizable(false)
                                .set_position(pos);
                            ctx.new_window(new_win);
                        })
                        .fix_width(100.0)
                        .fix_height(30.0),
                )
                .controller(Enter {
                    id_t: TimerToken::next(),
                    id_t2: TimerToken::next(),
                    locks: [false; 5],
                }),
        )
        .with_spacer(50.)
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
            show_screen_ui(img),
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
                .controller(ShortcutController { locks: [false; 5] }),
        )
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
    })
    .center();

    Flex::column().with_child(paint)
}

pub fn show_screen_ui(img: ImageBuf) -> impl Widget<AppState> {
    let image = Image::new(img.clone());
    let font = Font::try_from_vec(Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8])).unwrap();
    let points = Vec::<Point>::new();
    let mut path=druid::kurbo::BezPath::new();
    //let brush = BackgroundBrush::Color(druid::Color::rgb(255., 0., 0.));

    Flex::column()
        .with_child(Either::new(
            |data, _env| data.tool_window.tool == Tools::No,
            Flex::row()
                .with_child(Button::new("Resize").on_click(
                    |ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Resize;
                        data.resize = true;
                        data.annulla = false;
                        let width = data.img.width() as f64;
                        let height = data.img.height() as f64;

                        //println!("height:{},width:{}",height,width);
                        println!(
                            "height:{},width:{}",
                            data.tool_window.img_size.height, data.tool_window.img_size.width
                        );

                        let rect = druid::Rect::from_center_size(
                            data.tool_window.center,
                            data.tool_window.img_size,
                        );
                        data.rect.size = data.tool_window.img_size;
                        data.rect.start_point.replace(rect.origin());
                        data.rect
                            .end_point
                            .replace(Point::new(rect.max_x(), rect.max_y()));
                        data.rect.p2 = Some(Point::new(0., rect.max_y()));
                        data.rect.p3 = Some(Point::new(rect.max_x(), 0.));
                        data.tool_window.rect_stroke = 2.0;
                        data.tool_window.rect_transparency = 0.4;

                        //println!("rect start:{:?}",data.rect.start_point);
                        //ctx.children_changed();
                        //data.rect.start_point=None;
                        //data.rect.end_point=None;
                    },
                ))
                .with_child(Button::new("highlight").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Highlight
                    },
                ))
                .with_child(Button::new("ellipse").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Ellipse
                    },
                ))
                .with_child(Button::new("arrow").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Arrow
                    },
                ))
                .with_child(Button::new("text").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Text;
                    },
                ))
                .with_child(Button::new("redact").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Redact
                    },
                ))
                .with_child(Button::new("random").on_click(
                    |_ctx: &mut EventCtx, data: &mut AppState, _env| {
                        data.tool_window.tool = Tools::Random
                    },
                ))
                .with_child(
                    DropdownSelect::new(vec![
                        ("Red", Color::RED),
                        ("Green", Color::GREEN),
                        ("Black", Color::BLACK),
                        ("Blue", Color::BLUE),
                        ("Orange", Color::rgb8(211, 84, 0)),
                        ("Grey", Color::GRAY),
                    ])
                    .align_left()
                    .lens(AppState::color),
                ),Either::new(|data, _env| data.tool_window.tool == Tools::Text, 
                    Flex::row()
                        .with_child(
                            Button::new("salva").on_click(move |_ctx, data: &mut AppState, _env| {

                                if let Some(point)=data.tool_window.text_pos{

                                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::from_vec(
                                        data.img.width() as u32,
                                        data.img.height() as u32,
                                    data.tool_window.img.clone().unwrap().raw_pixels().to_vec()).unwrap();
            
                                    let color = data.color.as_rgba8();
            
                                    let strings=data.text.lines();
            
                                    let mut deref=5.;
                                    for s in strings{
                                        imageproc::drawing::draw_text_mut(
                                            &mut image,
                                            Rgba([color.0, color.1, color.2, 255]),
                                            ((point.x-data.tool_window.origin.x)*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                                            ((point.y+deref-data.tool_window.origin.y)*(data.img.height() as f64/data.tool_window.img_size.height)) as i32,
                                            rusttype::Scale{x:21.*(data.img.width() as f64/data.tool_window.img_size.width) as f32,y:22.*(data.img.height() as f64/data.tool_window.img_size.height) as f32},
                                            &font.clone(),
                                            s,
                                        );
    
                                        deref=deref+25.;
                                    }
                                    
                                    data.tool_window.img=Some(ImageBuf::from_raw(
                                        image.clone().into_raw(),
                                        druid::piet::ImageFormat::RgbaPremul,
                                        image.clone().width() as usize,
                                        image.clone().height() as usize,
                                    ));
                                    data.img = data.tool_window.img.clone().unwrap();
                                    data.tool_window.text_pos=None;
                                }
                                
                                data.text = "".to_string();
                                data.tool_window.tool = Tools::No;
                            }),
                        )
                        .with_child(
                            Button::new("annulla").on_click(|_ctx, data: &mut AppState, _env| {
                                data.text="".to_string();
                                data.tool_window.text_pos=None;
                                data.tool_window.img = Some(data.img.clone());
                                data.tool_window.tool = Tools::No;
                            }),
                        )
                        .with_child(
                            TextBox::multiline()
                                .with_placeholder("scrivi qui")
                                .fix_width(300.)
                                .fix_height(50.)
                                .lens(AppState::text)
                        ),
                        Flex::row()
                        .with_child( 
                            Button::new("salva").on_click(move |ctx, data: &mut AppState, env| {
                                match data.tool_window.tool {
                                    Tools::Resize => {
                                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
                                        ImageBuffer::from_vec(
                                            data.img.width() as u32,
                                            data.img.height() as u32,
                                            data.tool_window.img.clone().unwrap().raw_pixels().to_vec(),
                                        )
                                        .unwrap();
                                    let im = image.sub_image(
                                        ((data.rect.start_point.unwrap().x - data.tool_window.origin.x)
                                            * (data.img.width() as f64
                                                / data.tool_window.img_size.width))
                                            as u32,
                                        ((data.rect.start_point.unwrap().y - data.tool_window.origin.y)
                                            * (data.img.height() as f64
                                                / data.tool_window.img_size.height))
                                            as u32,
                                        (data.rect.size.width * data.img.width() as f64
                                            / data.tool_window.img_size.width)
                                            as u32,
                                        (data.rect.size.height * data.img.height() as f64
                                            / data.tool_window.img_size.height)
                                            as u32,
                                    );
                                    let imm = im.to_image();

                                    data.tool_window.img = Some(ImageBuf::from_raw(
                                        imm.clone().into_raw(),
                                        druid::piet::ImageFormat::RgbaPremul,
                                        (data.rect.size.width * data.img.width() as f64
                                            / data.tool_window.img_size.width)
                                            as usize,
                                        (data.rect.size.height * data.img.height() as f64
                                            / data.tool_window.img_size.height)
                                            as usize,
                                    ));

                                    data.img = data.tool_window.img.clone().unwrap();

                                    let width = data.img.width() as f64;
                                    let height = data.img.height() as f64;

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
                                    Tools::Ellipse => {
                                        data.img = data.tool_window.img.clone().unwrap();
                                    }
                                    Tools::Highlight => {
                                        data.color = data.color.with_alpha(1.);
                                        data.img = data.tool_window.img.clone().unwrap();
                                    }
                                    _ => {}
                                }

                                data.tool_window.rect_stroke = 0.0;
                                data.tool_window.rect_transparency = 0.;
                                data.tool_window.tool = Tools::No;
                            })
                        )
                        .with_child(
                            Button::new("annulla").on_click(|ctx, data: &mut AppState, env| {
                                match data.tool_window.tool {
                                    Tools::Resize => {
                                        data.tool_window.rect_stroke = 0.;
                                        data.tool_window.rect_transparency = 0.;
                                    }
                                    Tools::Ellipse => {}
                                    Tools::Highlight => {
                                        data.color = data.color.with_alpha(1.);
                                    }
                                    Tools::Random=>{
                                        println!("random");
                                    }
                                    _ => {}
                                }
                                data.tool_window.img = Some(data.img.clone());
                                data.tool_window.tool = Tools::No;
                            })
                        )
                        
                )
            /*Flex::row()
                .with_child(
                    Button::new("salva").on_click(|ctx, data: &mut AppState, env| {
                        match data.tool_window.tool {
                            Tools::Resize => {
                                let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
                                    ImageBuffer::from_vec(
                                        data.img.width() as u32,
                                        data.img.height() as u32,
                                        data.tool_window.img.clone().unwrap().raw_pixels().to_vec(),
                                    )
                                    .unwrap();
                                /*
                                println!("rect:start:{},end{}",data.rect.start_point.unwrap(),data.rect.end_point.unwrap());
                                println!("tool_window{}",data.tool_window.origin);
                                println!("img:width:{},height:{}",data.img.width(),data.img.height());
                                println!("start point:x:{},y:{}",((data.rect.start_point.unwrap().x-data.tool_window.origin.x)*(data.img.width() as f64/data.tool_window.img_size.width)) as u32,((data.rect.start_point.unwrap().y-data.tool_window.origin.y)*(data.img.height() as f64/data.tool_window.img_size.height)) as u32);
                                println!("subimage dimensions:width:{},height:{}",(data.rect.size.width*data.img.width() as f64/data.tool_window.img_size.width) as u32,(data.rect.size.height*data.img.height() as f64/data.tool_window.img_size.height) as u32);
                                //println!("x:{},width:{}",(data.rect.start_point.unwrap().x-data.tool_window.origin.x),(data.rect.size.width*data.img.width() as f64/data.tool_window.img_size.width));
                                */
                                let im = image.sub_image(
                                    ((data.rect.start_point.unwrap().x - data.tool_window.origin.x)
                                        * (data.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        as u32,
                                    ((data.rect.start_point.unwrap().y - data.tool_window.origin.y)
                                        * (data.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        as u32,
                                    (data.rect.size.width * data.img.width() as f64
                                        / data.tool_window.img_size.width)
                                        as u32,
                                    (data.rect.size.height * data.img.height() as f64
                                        / data.tool_window.img_size.height)
                                        as u32,
                                );
                                let imm = im.to_image();

                                data.tool_window.img = Some(ImageBuf::from_raw(
                                    imm.clone().into_raw(),
                                    druid::piet::ImageFormat::RgbaPremul,
                                    (data.rect.size.width * data.img.width() as f64
                                        / data.tool_window.img_size.width)
                                        as usize,
                                    (data.rect.size.height * data.img.height() as f64
                                        / data.tool_window.img_size.height)
                                        as usize,
                                ));

                                data.img = data.tool_window.img.clone().unwrap();

                                let width = data.img.width() as f64;
                                let height = data.img.height() as f64;

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
                            Tools::Ellipse => {
                                data.img = data.tool_window.img.clone().unwrap();
                            }
                            Tools::Text => {
                                data.img = data.tool_window.img.clone().unwrap();
                                data.tool_window.text = "".to_string();
                            }
                            Tools::Highlight => {
                                data.color = data.color.with_alpha(1.);
                                data.img = data.tool_window.img.clone().unwrap();
                            }
                            _ => {}
                        }

                        data.tool_window.rect_stroke = 0.0;
                        data.tool_window.rect_transparency = 0.;
                        data.tool_window.tool = Tools::No;
                    }),
                )
                .with_child(
                    Button::new("annulla").on_click(|ctx, data: &mut AppState, env| {
                        match data.tool_window.tool {
                            Tools::Resize => {
                                data.tool_window.rect_stroke = 0.;
                                data.tool_window.rect_transparency = 0.;
                            }
                            Tools::Ellipse => {}
                            Tools::Text => {
                                data.tool_window.text = "".to_string();
                            }
                            Tools::Highlight => {
                                data.color = data.color.with_alpha(1.);
                            }
                            Tools::Random => {
                                println!("random");
                            }
                            _ => {}
                        }
                        data.tool_window.img = Some(data.img.clone());
                        data.tool_window.tool = Tools::No;
                    }),
                ),*/
        ))
        .with_child(
            ZStack::new(
                SizedBox::new(
                    //image
                    Painter::new(|ctx, data: &AppState, env| {
                        //println!("height:{},width:{}",height,width);
                        //println!("image:height:{},width:{}",height,width);

                        let image = ctx
                            .make_image(
                                data.img.width(),
                                data.img.height(),
                                data.tool_window.img.clone().unwrap().raw_pixels(),
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
                .width(500.)
                .height(312.5)
                .background(BackgroundBrush::Color(druid::Color::rgb(255., 0., 0.))),
            )
            .with_centered_child(
                Painter::new(move|ctx, data: &AppState, env| {
                    match data.tool_window.tool {
                        Tools::Resize => {
                            if let (Some(start), Some(end)) =
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

                                ctx.fill(
                                    shape,
                                    &Color::rgba(
                                        color.0,
                                        color.1,
                                        color.2,
                                        data.selection_transparency,
                                    ),
                                );
                            }
                        }
                        Tools::Arrow=>{
                            if let(Some(start),Some(end))=(data.tool_window.shape.start_point,data.tool_window.shape.end_point){
                                
                                let color = data.color.as_rgba();

                                let body = druid::kurbo::Line::new(start, end);

                                ctx.stroke(
                                    body,
                                    &Color::rgba(color.0, color.1, color.2, data.tool_window.rect_transparency),
                                    10.,
                                );

                                let cos = 0.866;
                                let sin = 0.500;
                                let dx=end.x-start.x;
                                let dy=end.y-start.y;
                                let end1=druid::Point::new(end.x-(dx*cos+dy*-sin)*2./5.,end.y-(dx*sin+dy*cos)*2./5.);
                                let end2=druid::Point::new(end.x-(dx*cos+dy*sin)*2./5.,end.y-(dx*-sin+dy*cos)*2./5.);

                                ctx.stroke(
                                    druid::kurbo::Line::new(end,end1),
                                    &Color::rgba(color.0, color.1, color.2, data.tool_window.rect_transparency),
                                    10.,
                                );

                                ctx.stroke(
                                    druid::kurbo::Line::new(end,end2),
                                    &Color::rgba(color.0, color.1, color.2, data.tool_window.rect_transparency),
                                    10.,
                                );

                                /*
                                let h=((end.x-start.x).powi(2)+(end.y-start.y).powi(2)).sqrt();

                                let x=h*((2. as f64).sqrt()/2.);

                                let y=h*((2. as f64).sqrt()/2.);
                                */

                                /*
                                if start.x>end.x{
                                    if start.x-end.x<20.{
                                        ctx.stroke(
                                            druid::kurbo::Line::new(end,druid::Point::new(end.x-start.x, end.y)),
                                            &Color::rgba(color.0, color.1, color.2, color.3),
                                            10.,
                                        );
                                    }else {
                                        ctx.stroke(
                                            druid::kurbo::Line::new(end,druid::Point::new(end.x-20., end.y)),
                                            &Color::rgba(color.0, color.1, color.2, color.3),
                                            10.,
                                        );
                                    }
                                }else {
                                    if start.x-end.x<20.{
                                        ctx.stroke(
                                            druid::kurbo::Line::new(end,druid::Point::new(end.x-start.x, end.y)),
                                            &Color::rgba(color.0, color.1, color.2, color.3),
                                            10.,
                                        );
                                    }else {
                                        ctx.stroke(
                                            druid::kurbo::Line::new(end,druid::Point::new(end.x-20., end.y)),
                                            &Color::rgba(color.0, color.1, color.2, color.3),
                                            10.,
                                        );
                                    }
                                }
                                */
                                
                            }
                        }
                        Tools::Highlight => {
                            if let (Some(start), Some(end)) = (
                                data.tool_window.shape.start_point,
                                data.tool_window.shape.end_point,
                            ) {
                                //println!("highlight");
                                let color = data.color.as_rgba();

                                let shape = druid::kurbo::Line::new(start, end);

                                ctx.stroke(
                                    shape,
                                    &Color::rgba(color.0, color.1, color.2, color.3),
                                    10.,
                                );
                            }
                        }
                        Tools::Random => {
                            if let Some(point) = data.tool_window.random_point {
                                let color = data.color.as_rgba();
                                if path.is_empty(){
                                    path.push(druid::kurbo::PathEl::MoveTo(point));
                                    path.push(druid::kurbo::PathEl::LineTo(point));
                                }else {
                                    path.push(druid::kurbo::PathEl::LineTo(point));
                                }
                                

                                if path.is_empty(){
                                    //println!("vuoto");
                                }
                                if path.is_finite(){
                                    //println!("finite");
                                }
                                if path.is_nan(){
                                    println!("nan");
                                }

                                //println!("{:?}",path);

                                //ctx.fill(path.clone(), &Color::rgba(color.0, color.1, color.2, color.3));
                                ctx.stroke(path.clone(), &Color::rgba(color.0, color.1, color.2, color.3), 10.);
                                //ctx.fill(druid::kurbo::Circle::new(point, 10.),&Color::rgba(color.0, color.1, color.2, color.3));
                                /*ctx.fill_even_odd(
                                    shape,
                                    &Color::rgba(color.0, color.1, color.2, color.3),
                                )*/

                                if color.3==0.0{
                                    path=druid::kurbo::BezPath::new();
                                }
                            }
                        }
                        Tools::Text=>{

                            if let Some(point) =data.tool_window.text_pos {

                                let mut a=druid::text::TextLayout::new();

                                a.set_text_color(data.color);
                                a.set_font(
                                    druid::text::FontDescriptor{family:druid::text::FontFamily::SANS_SERIF,
                                        size:20.,
                                        weight:druid::text::FontWeight::NORMAL,
                                        style:druid::text::FontStyle::Regular,
                                });

                                a.set_text(data.text.clone());

                                a.rebuild_if_needed(ctx.text(), env);

                                a.draw(ctx, point);
                            }
                            

                        }
                        _ => {}
                    }
                })
                .center(),
            )
            .controller(ResizeController {
                points: points,
            }),
        )

    ////////////////////////roba di ciro
    /*let new_win = WindowDesc::new(
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
    )*/

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
