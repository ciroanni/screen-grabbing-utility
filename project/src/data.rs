use crate::ui::*;
use druid::Color;
use druid::{
    commands, widget::Controller, AppDelegate, Command, Cursor, Data, DelegateCtx, Env, Event,
    EventCtx, Handled, ImageBuf, Lens, LocalizedString, MouseEvent, Point, Selector, Size, Target,
    TimerToken, Widget, WindowDesc, WindowState,
};
use druid_shell::keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use druid_shell::piet::d2d::Bitmap;
use image::{ImageBuffer, Rgba};
use std::path::Path;
use std::time::Duration;
use rusttype::{Scale,Font};
//use imageproc::drawing::*;

pub const SHORTCUT: Selector = Selector::new("shortcut_selector");

#[derive(Clone, Data, PartialEq, Debug)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Pnm,
    Tiff,
    Tga,
    Dds,
    Bmp,
    Ico,
    Hdr,
    OpenExr,
    Farbfeld,
    Avif,
    Qoi,
}

impl ImageFormat {
    pub fn to_string(&self) -> String {
        match self {
            ImageFormat::Jpeg => ".jpeg".to_string(),
            ImageFormat::Png => ".png".to_string(),
            ImageFormat::Gif => ".gif".to_string(),
            ImageFormat::WebP => ".webp".to_string(),
            ImageFormat::Pnm => ".pnm".to_string(),
            ImageFormat::Tiff => ".tiff".to_string(),
            ImageFormat::Tga => ".tga".to_string(),
            ImageFormat::Dds => ".dds".to_string(),
            ImageFormat::Bmp => ".bmp".to_string(),
            ImageFormat::Ico => ".ico".to_string(),
            ImageFormat::Hdr => ".hdr".to_string(),
            ImageFormat::OpenExr => ".openexr".to_string(),
            ImageFormat::Farbfeld => ".farbfeld".to_string(),
            ImageFormat::Avif => ".avif".to_string(),
            ImageFormat::Qoi => ".qoi".to_string(),
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum Timer {
    Zero,
    ThreeSeconds,
    FiveSeconds,
    TenSeconds,
    Custom,
}

impl Timer {
    pub fn set_timer(&self) -> u64 {
        match self {
            Timer::Zero => 0,
            Timer::ThreeSeconds => 3,
            Timer::FiveSeconds => 5,
            Timer::TenSeconds => 10,
            _ => 0,
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum Tools{
    No,
    Resize,
    Ellipse,
    Arrow,
    Text,
    Highlight,
    Redact,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub name: String,
    pub selected_format: ImageFormat,
    pub shortcut: String,
    pub mods: u32,
    pub key: u32,
    pub from: Point,
    pub size: Point,
    pub scale: f32,
    pub rect: SelectionRectangle,
    pub is_full_screen: bool, //true --> end of area selection
    pub selection_transparency: f64,
    pub img: ImageBuf,
    pub cursor: CursorData,
    pub path: String,
    pub delay: Timer,
    pub resize: bool,
    pub annulla: bool,
    pub tool_window: AnnotationTools,
    pub color: Color,
}


impl AppState {
    pub fn new(scale: f32, img: ImageBuf) -> Self {
        let display_info = screenshots::DisplayInfo::all().expect("Err");

        let width = display_info[0].width as f32 * display_info[0].scale_factor;
        let height = display_info[0].height as f32 * display_info[0].scale_factor;

        AppState {
            name: "".to_string(),
            selected_format: ImageFormat::Jpeg,
            shortcut: "".to_string(),
            mods: Modifiers::ALT.bits(),
            key: Key::Character("s".to_string()).legacy_charcode(),
            from: Point { x: 0.0, y: 0.0 },
            size: Point {
                x: width as f64,
                y: height as f64,
            },
            scale,
            rect: SelectionRectangle::default(),
            is_full_screen: false,
            selection_transparency: 0.4,
            img,
            cursor: CursorData::new(Cursor::Arrow, None, false),
            path: ".".to_string(),
            delay: Timer::Zero,
            resize: false,
            annulla: true,
            tool_window:AnnotationTools::default(),
            color:Color::rgba(0.,0.,0.,1.),
        }
    }

    pub fn screen(&mut self, ctx: &mut EventCtx) {
        let a = screenshots::DisplayInfo::all();

        let display_info = match a {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };
        let b = screenshots::Screen::new(&display_info[0]);

        let c;
        if self.is_full_screen {
            c = b.capture();
        } else {
            c = b.capture_area(
                (self.rect.start_point.unwrap().x as f32 * self.scale) as i32,
                (self.rect.start_point.unwrap().y as f32 * self.scale) as i32,
                (self.rect.size.width as f32 * self.scale) as u32,
                (self.rect.size.height as f32 * self.scale) as u32,
            );
            //self.rect = SelectionRectangle::default();
        }

        let image = match c {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        self.img = ImageBuf::from_raw(
            image.clone().into_raw(),
            druid::piet::ImageFormat::RgbaPremul,
            image.clone().width() as usize,
            image.clone().height() as usize,
        );

        self.tool_window.img=Some(self.img.clone());

        let width = self.img.width() as f64;
        let height = self.img.height() as f64;

        if width>=self.tool_window.width{
            if height>=self.tool_window.height{
                if height-self.tool_window.height>width-self.tool_window.width{
                    self.tool_window.img_size.height=self.tool_window.height;
                    self.tool_window.img_size.width=width*(self.tool_window.height/height);
                }else {
                    self.tool_window.img_size.width=self.tool_window.width;
                    self.tool_window.img_size.height=height*(self.tool_window.width/width);
                }
            }else {
                self.tool_window.img_size.width=self.tool_window.width;
                self.tool_window.img_size.height=height*(self.tool_window.width/width);
            }
        }else {
            if height>self.tool_window.height{
                self.tool_window.img_size.height=self.tool_window.height;
                self.tool_window.img_size.width=width*(self.tool_window.height/height);
            }else {
                if self.tool_window.height-height>self.tool_window.width-width{
                    self.tool_window.img_size.width=self.tool_window.width;
                    self.tool_window.img_size.height=height*(self.tool_window.width/width);
                }else {
                    self.tool_window.img_size.height=self.tool_window.height;
                    self.tool_window.img_size.width=width*(self.tool_window.height/height);
                }
            }
        }

        //println!("inizializzazione altezza:{},{}",self.tool_window.img_size.height,self.img.height());
        //println!("inizializzazione larghezza:{},{}",self.tool_window.img_size.width,self.img.width());

        self.tool_window.origin=druid::Point::new(
            self.tool_window.center.x-(self.tool_window.img_size.width/2.),
            self.tool_window.center.y-(self.tool_window.img_size.height/2.),
        );


        let window = WindowDesc::new(build_ui(self.img.clone()))
            .menu(make_menu)
            .title("Screen grabbing")
            .window_size((1000., 500.))
            .set_position(Point::new(0., 0.));
        ctx.new_window(window);
        //*self = AppState::new(self.scale, self.img.clone());
        self.selection_transparency = 0.4;

        //ctx.window().close();
    }

    pub fn set_default_name(&mut self) {
        let first: String;
        if self.name.is_empty() {
            first = String::from("screenshot");
        } else {
            first = self.name.clone();
        }

        let mut str3 = format!("{}{}", first, self.selected_format.to_string());

        let mut index = 0;
        loop {
            if !Path::new(&(self.path.clone() + "\\" + &str3)).exists() {
                //println!("{}", str3);
                break;
            } else {
                index += 1;
                str3 = format!(
                    "{}{}{}",
                    first,
                    index.to_string(),
                    self.selected_format.to_string()
                );
            }
        }
        if index == 0 {
            self.name = first;
        } else {
            self.name = format!("{}{}", first, index.to_string());
        }
    }

    pub fn save(&mut self) {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            self.img.width() as u32,
            self.img.height() as u32,
            self.img.raw_pixels().to_vec(),
        )
        .unwrap();
        self.set_default_name();
        image
            .save_with_format(
                self.path.clone() + "\\" + &self.name + &self.selected_format.to_string(),
                image::ImageFormat::Png,
            )
            .expect("Error saving");
        self.name = "".to_string();
        self.rect = SelectionRectangle::default();
    }

    pub fn save_as(&mut self, path: &Path) {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            self.img.width() as u32,
            self.img.height() as u32,
            self.img.raw_pixels().to_vec(),
        )
        .unwrap();
        image
            .save_with_format(path, image::ImageFormat::Png)
            .expect("Error saving");
        self.rect = SelectionRectangle::default();
    }
}

#[derive(Clone, Data, PartialEq, Lens, Debug)]
pub struct SelectionRectangle {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub p2: Option<Point>,
    pub p3: Option<Point>,
    pub size: Size,
}

impl Default for SelectionRectangle {
    fn default() -> Self {
        SelectionRectangle {
            start_point: None, //p1
            end_point: None,   // p4
            p2: None,
            p3: None,
            size: Size::ZERO,
        }
    }
}
/*
start = p1 .____. p2
           |    |
        p3 .____. p4 = end

 */

 #[derive(Clone, Data, PartialEq, Lens, Debug)]
 pub struct SelectionEllipse {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub center: Option<Point>,
    pub radii: Option<druid::Vec2>
}

impl Default for SelectionEllipse {
    fn default() -> Self {
        SelectionEllipse {
            start_point: None, //p1
            end_point: None,   // p4
            center: None,
            radii:None,
        }
    }
}

#[derive(Clone, Data, Lens, Debug)]
pub struct AnnotationTools {
    pub tool: Tools,
    pub center: druid::Point,
    pub origin: druid::Point,
    pub width:f64,
    pub height:f64,
    pub img_size: Size,
    pub rect_stroke: f64,
    pub rect_transparency: f64,
    pub ellipse: SelectionEllipse,
    pub img: Option<ImageBuf>,
    pub text: String,
    pub text_pos: druid::Point,
}

impl Default for AnnotationTools {
   fn default() -> Self {
       AnnotationTools {
        tool: Tools::No,
        center: druid::Point::new(250., 156.25),
        origin: druid::Point::new(0.,0.),
        width:500.,
        height:312.5,
        img_size:Size::ZERO,
        rect_stroke:0.0,
        rect_transparency:0.0,
        ellipse:SelectionEllipse::default(),
        img:None,
        text:"".to_string(),
        text_pos:druid::Point::new(250.,156.25),
        //text_font:Font::try_from_vec(Vec::from(include_bytes!("DejaVuSans.ttf") as &[u8])).unwrap(),
       }
   }
}


#[derive(Clone, Data, PartialEq, Lens, Debug)]
pub struct CursorData {
    typ: Cursor,
    over: Option<Direction>,
    down: bool,
}

impl CursorData {
    pub fn new(typ: Cursor, over: Option<Direction>, down: bool) -> Self {
        Self { typ, over, down }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

//Controller to take screen after the custom shortcut
pub struct Enter {
    pub id_t: TimerToken,
    pub id_t2: TimerToken,
}

impl<W: Widget<AppState>> Controller<AppState, W> for Enter {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        ctx.request_focus();
        match event {
            Event::KeyDown(key) => {
                let keyboard_event = KeyboardEvent {
                    state: key.state,
                    key: key.key.clone(),
                    code: key.code,
                    location: key.location,
                    modifiers: key.mods.raw(),
                    repeat: key.repeat,
                    is_composing: true,
                };

                ShortcutMatcher::from_event(keyboard_event).shortcut(
                    Modifiers::from_bits(data.mods).expect("Not a modifier"),
                    Key::Character(char::from_u32(data.key).expect("Not a char").to_string()),
                    || {
                        println!("shortcut matcher");
                        self.id_t = ctx.request_timer(Duration::from_millis(100));
                        ctx.window()
                            .clone()
                            .set_window_state(WindowState::Minimized);
                    },
                );
            }
            Event::Timer(id) => {
                if self.id_t == *id {
                    self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                    self.id_t = TimerToken::next();
                } else if self.id_t2 == *id {
                    data.screen(ctx);
                    ctx.window().close();
                }
            }
            _ => child.event(ctx, event, data, env),
        }

        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

//Controller to save custom shortcut
pub struct ShortcutController;

impl<W: Widget<AppState>> Controller<AppState, W> for ShortcutController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        if let Event::KeyDown(key) = event {
            let keyboard_event = KeyboardEvent {
                state: key.state,
                key: key.key.clone(),
                code: key.code,
                location: key.location,
                modifiers: key.mods.raw(),
                repeat: key.repeat,
                is_composing: true,
            };
            println!("{:?} {:?}", keyboard_event.modifiers, keyboard_event.key);

            data.mods = keyboard_event.modifiers.bits();
            data.key = keyboard_event.key.legacy_charcode();
        }

        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

//Controller for the click and drag motion
pub struct AreaController {
    pub id_t: TimerToken,
    pub id_t2: TimerToken,
    pub flag: bool,
}

impl<W: Widget<AppState>> Controller<AppState, W> for AreaController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        //if !data.resize {
        if !self.flag {
            data.is_full_screen = false;
            match event {
                Event::MouseDown(mouse_button) => {
                    let mouse_down = MouseEvent {
                        pos: mouse_button.pos,
                        window_pos: mouse_button.window_pos,
                        buttons: mouse_button.buttons,
                        mods: mouse_button.mods,
                        count: mouse_button.count,
                        focus: mouse_button.focus,
                        button: mouse_button.button,
                        wheel_delta: mouse_button.wheel_delta,
                    };
                    data.from = mouse_down.pos;
                    data.rect.start_point = Some(mouse_button.pos);
                    data.rect.end_point = None;
                }
                Event::MouseUp(mouse_button) => {
                    let mouse_up = MouseEvent {
                        pos: mouse_button.pos,
                        window_pos: mouse_button.window_pos,
                        buttons: mouse_button.buttons,
                        mods: mouse_button.mods,
                        count: mouse_button.count,
                        focus: mouse_button.focus,
                        button: mouse_button.button,
                        wheel_delta: mouse_button.wheel_delta,
                    };
                    //data.rect.end_point = Some(mouse_button.pos);
                    let r = druid::Rect::from_points(data.from, mouse_button.pos);

                    // aggiusto i punti
                    data.rect.start_point = Some(r.origin());
                    data.rect.p2 = Some(Point::new(r.max_x(), r.min_y()));
                    data.rect.p3 = Some(Point::new(r.min_x(), r.max_y()));
                    data.rect.end_point = Some(Point::new(r.max_x(), r.max_y()));
                    data.rect.size = r.size();
                    data.selection_transparency = 0.0;

                    match data.delay {
                        Timer::Zero => self.id_t = ctx.request_timer(Duration::from_millis(100)),
                        _ => {
                            self.id_t =
                                ctx.request_timer(Duration::from_secs(data.delay.set_timer()))
                        }
                    }

                    //ctx.window().clone().hide();
                }
                Event::MouseMove(mouse_button) => {
                    if !data.rect.start_point.is_none() {
                        data.rect.end_point = Some(mouse_button.pos);
                    }
                }
                Event::Timer(id) => {
                    if self.id_t == *id {
                        //ctx.window().clone().show();
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        self.id_t = TimerToken::next();
                    } else if self.id_t2 == *id {
                        data.screen(ctx);
                        //data.resize = true;
                        ctx.window().close();
                    }
                }
                _ => child.event(ctx, event, data, env),
            }
        } else {
            match event {
                Event::Timer(id) => {
                    if self.id_t == *id {
                        ctx.window().clone().set_window_state(WindowState::Restored);
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        self.id_t = TimerToken::next();
                    } else if self.id_t2 == *id {
                        data.screen(ctx);
                        ctx.window().close();
                    }
                }
                _ => {
                    data.is_full_screen = true;
                    match data.delay {
                        Timer::Zero => self.id_t = ctx.request_timer(Duration::from_millis(100)),
                        _ => {
                            self.id_t =
                                ctx.request_timer(Duration::from_secs(data.delay.set_timer()))
                        }
                    }

                    ctx.window()
                        .clone()
                        .set_window_state(WindowState::Minimized)
                }
            }
        }
        //}
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

//Controller to resize screening area

pub struct ResizeController{
    pub text_font:Font<'static>,
}

impl<W: Widget<AppState>> Controller<AppState, W> for ResizeController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {

        /*
        println!("{:?}",event);
        if data.resize {
            let width = data.rect.size.width;
            let height = data.rect.size.height;
            if data.rect.size.width > 500. {
                if data.rect.size.height > 312.5 {
                    data.rect.size.width = 500.;
                    data.rect.size.height = height / (width / 500.);
                    if data.rect.size.height > 312.5 {
                        data.rect.size.height = 312.5;
                        data.rect.size.width = width / (height / 312.5);
                    }
                } else {
                    data.rect.size.width = 500.;
                    data.rect.size.height = height / (width / 500.);
                }
            } else {
                data.rect.size.height = 312.5;
                data.rect.size.width = data.rect.size.width / (height / 312.5);
            }
            let rect = druid::Rect::from_center_size(Point::new(250., 156.25), data.rect.size);
            data.rect.start_point.replace(rect.origin());
            data.rect
                .end_point
                .replace(Point::new(rect.max_x(), rect.max_y()));
            data.resize = false;
        }
        */

        match data.tool_window.tool {
            Tools::Resize=>{
                if let Event::MouseDown(_mouse_button) = event {
                    match data.cursor.over {
                        Some(_) => {
                            data.cursor.down = true; // è sopra il bordo, sto premendo
                        }
                        None => return, // non è sopra il bordo
                    }
                } else if let Event::MouseUp(_mouse_button) = event {
                    //data.selection_transparency = 0.0;
                    data.cursor.down = false;
                    data.rect.size = druid::Rect::from_points(
                        data.rect.start_point.unwrap(),
                        data.rect.end_point.unwrap(),
                    )
                    .size();
                } else if let Event::MouseMove(mouse_button) = event {
                    let mouse_button = MouseEvent {
                        pos: mouse_button.pos,
                        window_pos: mouse_button.window_pos,
                        buttons: mouse_button.buttons,
                        mods: mouse_button.mods,
                        count: mouse_button.count,
                        focus: mouse_button.focus,
                        button: mouse_button.button,
                        wheel_delta: mouse_button.wheel_delta,
                    };
        
                    let rect = druid::Rect::from_points(
                        data.rect.start_point.unwrap(),
                        data.rect.end_point.unwrap(),
                    );
                    /* let rect = druid::Rect::from_center_size(
                        Point::new(250., 156.25),
                        Size::new(data.rect.size.width, data.rect.size.height),
                    ); */
        
                    //sposto senza premere
                    if data.cursor.down == false {
                        // cambia cursore -> diagonale
                        if (mouse_button.pos.x - rect.min_x()).abs() <= 10.
                            && (mouse_button.pos.y - rect.min_y()).abs() <= 10.
                        {
                            data.cursor.typ = Cursor::Crosshair;
                            data.cursor.over = Some(Direction::UpLeft);
                        } else if (mouse_button.pos.x - rect.max_x()).abs() <= 10.
                            && (mouse_button.pos.y - rect.max_y()).abs() <= 10.
                        {
                            data.cursor.typ = Cursor::Crosshair;
                            data.cursor.over = Some(Direction::DownRight);
                        }
                        // cambia cursore -> antidiagonale
                        else if (mouse_button.pos.x - rect.min_x()).abs() <= 10.
                            && (mouse_button.pos.y - rect.max_y()).abs() <= 10.
                        {
                            data.cursor.typ = Cursor::Crosshair;
                            data.cursor.over = Some(Direction::DownLeft);
                        } else if (mouse_button.pos.y - rect.min_y()).abs() <= 10.
                            && (mouse_button.pos.x - rect.max_x()).abs() <= 10.
                        {
                            data.cursor.typ = Cursor::Crosshair;
                            data.cursor.over = Some(Direction::UpRight);
                        } else if (mouse_button.pos.x - rect.min_x()).abs() <= 10. {
                            if mouse_button.pos.y > rect.min_y() && mouse_button.pos.y < rect.max_y() {
                                // cambia cursore -> sinistra
                                data.cursor.typ = Cursor::ResizeLeftRight;
                                data.cursor.over = Some(Direction::Left);
                            }
                        } else if (mouse_button.pos.x - rect.max_x()).abs() <= 10. {
                            if mouse_button.pos.y > rect.min_y() && mouse_button.pos.y < rect.max_y() {
                                // cambia cursore -> destra
                                data.cursor.typ = Cursor::ResizeLeftRight;
                                data.cursor.over = Some(Direction::Right);
                            }
                        } else if (mouse_button.pos.y - rect.max_y()).abs() <= 10. {
                            if mouse_button.pos.x > rect.min_x() && mouse_button.pos.x < rect.max_x() {
                                // cambia cursore -> verticale
                                data.cursor.typ = Cursor::ResizeUpDown;
                                data.cursor.over = Some(Direction::Down);
                            }
                        } else if (mouse_button.pos.y - rect.min_y()).abs() <= 10. {
                            if mouse_button.pos.x > rect.min_x() && mouse_button.pos.x < rect.max_x() {
                                // cambia cursore -> verticale
                                data.cursor.typ = Cursor::ResizeUpDown;
                                data.cursor.over = Some(Direction::Up);
                            }
                        } else {
                            data.cursor.typ = Cursor::Arrow;
                            data.cursor.over = None;
                        }
                        ctx.set_cursor(&data.cursor.typ);
                    }
                    //sposto premendo
                    else if data.cursor.down == true {
                        match data.cursor.over {
                            Some(Direction::Up) => {
                                if mouse_button.pos.y < data.rect.p3.unwrap().y - 10. && mouse_button.pos.y>(156.25-data.tool_window.img_size.height/2.){
                                    data.rect.start_point.replace(
                                        Point::new(
                                            data.rect.start_point.unwrap().x,
                                            mouse_button.pos.y,
                                        ));
                                    data.rect
                                        .p2
                                        .replace(Point::new(data.rect.p2.unwrap().x, mouse_button.pos.y));
                                }
                            }
                            Some(Direction::Down) => {
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10. && mouse_button.pos.y<(156.25+data.tool_window.img_size.height/2.){
                                    data.rect.end_point.replace(
                                        Point::new(
                                            data.rect.end_point.unwrap().x,
                                            mouse_button.pos.y,
                                        ));
                                    data.rect
                                        .p3
                                        .replace(Point::new(data.rect.p3.unwrap().x, mouse_button.pos.y));
                                }
                            }
                            Some(Direction::Left) => {
                                if mouse_button.pos.x < data.rect.p2.unwrap().x - 10. && mouse_button.pos.x>(250.-data.tool_window.img_size.width/2.) {
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                    data.rect
                                        .p3
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p3.unwrap().y));
                                }
                            }
                            Some(Direction::Right) => {
                                if mouse_button.pos.x > data.rect.start_point.unwrap().x + 10. && mouse_button.pos.x<(250.+data.tool_window.img_size.width/2.){
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                    data.rect
                                        .p2
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p2.unwrap().y));
                                }
                            }
                            Some(Direction::UpLeft) => {
                                if mouse_button.pos.x < data.rect.end_point.unwrap().x - 10. && mouse_button.pos.x>(250.-data.tool_window.img_size.width/2.){
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                    data.rect
                                        .p3
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p3.unwrap().y));
                                }
                                if mouse_button.pos.y < data.rect.end_point.unwrap().y - 10. && mouse_button.pos.y>(156.25-data.tool_window.img_size.height/2.) {
                                    data.rect.start_point.replace(Point::new(
                                        data.rect.start_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect
                                        .p2
                                        .replace(Point::new(data.rect.p2.unwrap().x, mouse_button.pos.y));
                                }
                            }
                            Some(Direction::UpRight) => {
                                if mouse_button.pos.x > data.rect.p3.unwrap().x + 10. && mouse_button.pos.x<(250.+data.tool_window.img_size.width/2.){
                                    data.rect
                                        .p2
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p2.unwrap().y));
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y < data.rect.end_point.unwrap().y - 10. && mouse_button.pos.y>(156.25-data.tool_window.img_size.height/2.){
                                    data.rect
                                        .p2
                                        .replace(Point::new(data.rect.p2.unwrap().x, mouse_button.pos.y));
                                    data.rect.start_point.replace(Point::new(
                                        data.rect.start_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::DownLeft) => {
                                if mouse_button.pos.x < data.rect.end_point.unwrap().x - 10. && mouse_button.pos.x>(250.-data.tool_window.img_size.width/2.){
                                    data.rect
                                        .p3
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p3.unwrap().y));
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10. && mouse_button.pos.y<(156.25+data.tool_window.img_size.height/2.){
                                    data.rect
                                        .p3
                                        .replace(Point::new(data.rect.p3.unwrap().x, mouse_button.pos.y));
                                    data.rect.end_point.replace(Point::new(
                                        data.rect.end_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::DownRight) => {
                                if mouse_button.pos.x > data.rect.p3.unwrap().x + 10. && mouse_button.pos.x<(250.+data.tool_window.img_size.width/2.){
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                    data.rect
                                        .p2
                                        .replace(Point::new(mouse_button.pos.x, data.rect.p2.unwrap().y));
                                }
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10. && mouse_button.pos.y<(156.25+data.tool_window.img_size.height/2.) {
                                    data.rect.end_point.replace(Point::new(
                                        data.rect.end_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect
                                        .p3
                                        .replace(Point::new(data.rect.p3.unwrap().x, mouse_button.pos.y));
                                }
                            }
                            None => return, // non è sopra il bordo
                        }
                    }
                }
            },
            Tools::Ellipse=>{
                match event {
                    Event::MouseDown(mouse_button) => {
                        let mouse_down = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };
                        data.tool_window.ellipse.start_point=Some(mouse_down.pos);
                        data.tool_window.ellipse.end_point=Some(mouse_down.pos);

                        //println!("{:?}",data.tool_window.ellipse.end_point);
                        
                        data.selection_transparency=1.;
                    }
                    Event::MouseMove(mouse_button)=>{
                        let mouse_move = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };
        
                        data.tool_window.ellipse.end_point=Some(mouse_move.pos);
                        if let (Some(start), Some(end)) = (data.tool_window.ellipse.start_point, data.tool_window.ellipse.end_point) {
                            let radius1 = (start.x - end.x) / 2.;
                            let radius2 = (start.y - end.y) / 2.;
                            let c1 = end.x + radius1;
                            let c2 = end.y + radius2;
                            data.tool_window.ellipse.center = Some(druid::Point::new(c1, c2));
                            data.tool_window.ellipse.radii = Some(druid::Vec2::new(radius1.abs(), radius2.abs()));
                        }
                    }
                    Event::MouseUp(mouse_button)=>{
                        let mouse_up = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };

                        data.selection_transparency=0.;
                        
                        let mut image: ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::from_vec(
                            data.img.width() as u32,
                            data.img.height() as u32,
                        data.tool_window.img.clone().unwrap().raw_pixels().to_vec()).unwrap();


                        let color = data.color.as_rgba8();
                        let prova=imageproc::drawing::draw_filled_ellipse(
                            &mut image,
                            (((data.tool_window.ellipse.center.unwrap().x-data.tool_window.origin.x)*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                                ((data.tool_window.ellipse.center.unwrap().y-data.tool_window.origin.y)*(data.img.height() as f64/data.tool_window.img_size.height))  as i32),
                            (data.tool_window.ellipse.radii.unwrap().x*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                            (data.tool_window.ellipse.radii.unwrap().y*(data.img.height() as f64/data.tool_window.img_size.height)) as i32,
                            Rgba([color.0, color.1, color.2, 255]));

                        data.tool_window.img=Some(ImageBuf::from_raw(
                            prova.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            prova.clone().width() as usize,
                            prova.clone().height() as usize,
                        ));

                        data.tool_window.ellipse.start_point=None;
                        data.tool_window.ellipse.end_point=None;
                        data.tool_window.ellipse.center=None;
                        data.tool_window.ellipse.radii=None;
        /*
                        data.tool_window.ellipse.end_point=Some(mouse_up.pos);
                        //let a=image_canvas::layout::CanvasLayout::with_plane(data.img);
                        let mut image2: ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::from_vec(
                            data.img.width() as u32,
                            data.img.height() as u32,
                        data.img.raw_pixels().to_vec()).unwrap();
        
                        let prova=imageproc::drawing::draw_filled_ellipse(
                            &mut image2,
                            (mouse_up.pos.x as i32,mouse_up.pos.y as i32),
                            100,
                            100,
                            Rgba([255,0,0,255]));
        
                        
                        
                        data.img=ImageBuf::from_raw(
                            prova.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            prova.clone().width() as usize,
                            prova.clone().height() as usize,
                        );
*/
                        //data.img.to_image(ctx);
                        //data.tool_window.tool=Tools::No;
                    },
                    _=>{}
                }
            }
            Tools::Text=>{
                ctx.request_focus();
                match event {
                    Event::MouseDown(mouse_button) => {
                        let mouse_down = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };
                        data.tool_window.text_pos=mouse_down.pos;

                        //println!("{:?}",data.tool_window.ellipse.end_point);
                        
                        //data.selection_transparency=1.;
                    }
                    /*
                    Event::MouseMove(mouse_button)=>{
                        let mouse_move = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };
        
                        data.tool_window.ellipse.end_point=Some(mouse_move.pos);
                        if let (Some(start), Some(end)) = (data.tool_window.ellipse.start_point, data.tool_window.ellipse.end_point) {
                            let radius1 = (start.x - end.x) / 2.;
                            let radius2 = (start.y - end.y) / 2.;
                            let c1 = end.x + radius1;
                            let c2 = end.y + radius2;
                            data.tool_window.ellipse.center = Some(druid::Point::new(c1, c2));
                            data.tool_window.ellipse.radii = Some(druid::Vec2::new(radius1.abs(), radius2.abs()));
                        }
                    }
                    */
                    Event::KeyDown(key)=>{
                        let a="Enter".to_string();
                        if key.key.to_string()=="Enter"{
                            data.tool_window.text=format!("{}\n",data.tool_window.text);
                        }else {
                            if key.key.to_string()=="Backspace"{
                                data.tool_window.text.remove(data.tool_window.text.len()-1);
                                data.tool_window.img=Some(data.img.clone());
                            }else {
                                data.tool_window.text=format!("{}{}",data.tool_window.text,key.key.to_string());
                            }
                        }

                        //////////////////////////////come gestisco l'"invio"
                        /*
                        match key.key.to_string(){
                            "Enter".to_string()=>{},
                            _=>{},
                        }
                        */
                        if !key.mods.is_empty(){
                            println!("modifier");
                        }
                        

                        println!("{}",data.tool_window.text);

                        let mut image: ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::from_vec(
                            data.img.width() as u32,
                            data.img.height() as u32,
                        data.tool_window.img.clone().unwrap().raw_pixels().to_vec()).unwrap();

                        let color = data.color.as_rgba8();

                        let prova=imageproc::drawing::draw_text(
                            &mut image,
                            Rgba([color.0, color.1, color.2, 255]),
                            ((data.tool_window.text_pos.x-data.tool_window.origin.x)*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                            ((data.tool_window.text_pos.y-data.tool_window.origin.y)*(data.img.height() as f64/data.tool_window.img_size.height)-20.) as i32,
                            rusttype::Scale{x:50.,y:25.},
                            &self.text_font,
                            data.tool_window.text.as_str(),
                        );

                        data.tool_window.img=Some(ImageBuf::from_raw(
                            prova.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            prova.clone().width() as usize,
                            prova.clone().height() as usize,
                        ));
                        
                    }
                    /*
                    Event::MouseUp(mouse_button)=>{
                        let mouse_up = MouseEvent {
                            pos: mouse_button.pos,
                            window_pos: mouse_button.window_pos,
                            buttons: mouse_button.buttons,
                            mods: mouse_button.mods,
                            count: mouse_button.count,
                            focus: mouse_button.focus,
                            button: mouse_button.button,
                            wheel_delta: mouse_button.wheel_delta,
                        };


                        data.selection_transparency=0.;
                        
                        let mut image: ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::from_vec(
                            data.img.width() as u32,
                            data.img.height() as u32,
                        data.tool_window.img.clone().unwrap().raw_pixels().to_vec()).unwrap();


                        let prova=imageproc::drawing::draw_filled_ellipse(
                            &mut image,
                            (((data.tool_window.ellipse.center.unwrap().x-data.tool_window.origin.x)*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                                ((data.tool_window.ellipse.center.unwrap().y-data.tool_window.origin.y)*(data.img.height() as f64/data.tool_window.img_size.height))  as i32),
                            (data.tool_window.ellipse.radii.unwrap().x*(data.img.width() as f64/data.tool_window.img_size.width)) as i32,
                            (data.tool_window.ellipse.radii.unwrap().y*(data.img.height() as f64/data.tool_window.img_size.height)) as i32,
                            Rgba([255,0,0,255]));

                        data.tool_window.img=Some(ImageBuf::from_raw(
                            prova.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            prova.clone().width() as usize,
                            prova.clone().height() as usize,
                        ));

                        data.tool_window.ellipse.start_point=None;
                        data.tool_window.ellipse.end_point=None;
                        data.tool_window.ellipse.center=None;
                        data.tool_window.ellipse.radii=None;
                    },
                    */
                    _=>{}
                }
            }
            Tools::Highlight=>{

            }
            _=>{}
        }
        

        if !ctx.is_hot(){
            data.cursor.over=None;
            data.cursor.down=false;
        }
        if ctx.is_focused() && data.tool_window.tool!=Tools::Text{
            ctx.resign_focus();
            //ctx.focus_prev();
        }
        //}
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}
pub struct Delegate; //vedi main.rs
impl AppDelegate<AppState> for Delegate {
    //mi permette di gestire i comandi di show_save_panel e show_open_panel, rispettivamente infatti chiamano SAVE_FILE e OPEN_FILE
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
            if let Err(e) = std::fs::write(file_info.path(), &data.name[..]) {
                println!("Error writing file: {e}");
            } else {
                data.save_as(file_info.path());
                return Handled::Yes;
            }
        }
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match std::fs::read_dir(file_info.path()) {
                Ok(_s) => {
                    data.path = file_info.path().to_str().unwrap().to_string();
                }
                Err(e) => {
                    println!("Error opening folder: {e}");
                }
            }
            return Handled::Yes;
        }
        if cmd.is(SHORTCUT) {
            let new_win = WindowDesc::new(shortcut_ui())
                .title(LocalizedString::new("Shortcut"))
                .window_size((300.0, 200.0));
            ctx.new_window(new_win);
        }
        Handled::No
    }
}
