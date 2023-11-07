use crate::ui::*;
use crossbeam::channel::{bounded, Receiver as CrossReceiver, Sender as CrossSender};
use druid::Color;
use druid::{
    commands, widget::Controller, AppDelegate, Command, Cursor, Data, DelegateCtx, Env, Event,
    EventCtx, Handled, ImageBuf, Lens, LocalizedString, MouseEvent, Point, Selector, Size, Target,
    TimerToken, Widget, WindowDesc, WindowState,
};
use druid_shell::keyboard_types::Key;
use image::{ImageBuffer, Rgba};
use livesplit_hotkey::*;
use rusttype::Font;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
//use global_hotkey::*;
//use imageproc::drawing::*;

pub const SHORTCUT: Selector = Selector::new("shortcut_selector");

#[derive(Clone, PartialEq, Debug)]
pub struct MyModifier {
    pub modifier: livesplit_hotkey::Modifiers,
}
impl Data for MyModifier {
    fn same(&self, other: &Self) -> bool {
        return self.modifier == other.modifier;
    }
}

#[derive(Clone, PartialEq)]
pub struct MyKey {
    pub key: livesplit_hotkey::KeyCode,
}
impl Data for MyKey {
    fn same(&self, other: &Self) -> bool {
        return self.key == other.key;
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum ImageFormat {
    Jpeg,
}

impl ImageFormat {
    pub fn to_string(&self) -> String {
        match self {
            ImageFormat::Jpeg => ".jpeg".to_string(),
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum Timer {
    Zero,
    ThreeSeconds,
    FiveSeconds,
    TenSeconds,
}

impl Timer {
    pub fn set_timer(&self) -> u64 {
        match self {
            Timer::Zero => 0,
            Timer::ThreeSeconds => 3,
            Timer::FiveSeconds => 5,
            Timer::TenSeconds => 10,
        }
    }
}

#[derive(Clone, Data, PartialEq, Debug)]
pub enum Tools {
    No,
    Resize,
    Ellipse,
    Rectangle,
    Arrow,
    Text,
    Highlight,
    Random,
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub name: String,
    pub selected_format: ImageFormat,
    pub shortcut: String,
    #[data(ignore)]
    pub sender: Sender<(Hotkey, u32)>,
    #[data(ignore)]
    pub receiver: CrossReceiver<u32>,
    #[data(ignore)]
    pub full_mods: (
        livesplit_hotkey::Modifiers,
        livesplit_hotkey::Modifiers,
        livesplit_hotkey::Modifiers,
    ),
    pub full_mod1: MyModifier,
    pub full_mod2: MyModifier,
    pub full_mod3: MyModifier,
    pub full_k: String,
    #[data(ignore)]
    pub full_key: livesplit_hotkey::KeyCode,
    //pub full_id:u32,
    #[data(ignore)]
    pub area_mods: (
        livesplit_hotkey::Modifiers,
        livesplit_hotkey::Modifiers,
        livesplit_hotkey::Modifiers,
    ),
    pub area_mod1: MyModifier,
    pub area_mod2: MyModifier,
    pub area_mod3: MyModifier,
    pub area_k: String,
    #[data(ignore)]
    pub area_key: livesplit_hotkey::KeyCode,
    pub err: bool,
    //pub area_id:u32,
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
    pub text: String,
    pub pos: Point,
    pub num_display: usize,
    pub fill_shape: bool,
    pub color_picker: bool,
    pub edit: bool,
}

impl AppState {
    pub fn new(scale: f32, img: ImageBuf) -> Self {
        let display_info = screenshots::DisplayInfo::all().expect("Err");

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

        let (sender, receiver) = channel();
        let (send, recv) = bounded(1);
        /*
        let mut tasti1 = global_hotkey::hotkey::HotKey::new(Some(global_hotkey::hotkey::Modifiers::ALT), global_hotkey::hotkey::Code::KeyS);
        let mut tasti2 = global_hotkey::hotkey::HotKey::new(Some(global_hotkey::hotkey::Modifiers::ALT), global_hotkey::hotkey::Code::KeyG);

        let id1=tasti1.id();
        let id2=tasti2.id();
        */
        std::thread::spawn(|| {
            create_listener(receiver, send);
        });
        /*
                sender.send((tasti1,1));
                sender.send((tasti2,2));
        */
        AppState {
            name: "".to_string(),
            selected_format: ImageFormat::Jpeg,
            shortcut: "".to_string(),
            sender: sender,
            receiver: recv,
            full_mods: (
                livesplit_hotkey::Modifiers::ALT,
                livesplit_hotkey::Modifiers::empty(),
                livesplit_hotkey::Modifiers::empty(),
            ),
            full_mod1: MyModifier {
                modifier: livesplit_hotkey::Modifiers::ALT,
            },
            full_mod2: MyModifier {
                modifier: livesplit_hotkey::Modifiers::empty(),
            },
            full_mod3: MyModifier {
                modifier: livesplit_hotkey::Modifiers::empty(),
            },
            full_k: "S".to_string(),
            full_key: livesplit_hotkey::KeyCode::KeyS,
            //full_id:id1,
            area_mods: (
                livesplit_hotkey::Modifiers::ALT,
                livesplit_hotkey::Modifiers::empty(),
                livesplit_hotkey::Modifiers::empty(),
            ),
            area_mod1: MyModifier {
                modifier: livesplit_hotkey::Modifiers::ALT,
            },
            area_mod2: MyModifier {
                modifier: livesplit_hotkey::Modifiers::empty(),
            },
            area_mod3: MyModifier {
                modifier: livesplit_hotkey::Modifiers::empty(),
            },
            area_k: "G".to_string(),
            area_key: livesplit_hotkey::KeyCode::KeyG,
            err: false,
            //area_id:id2,
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
            tool_window: AnnotationTools::default(),
            color: Color::rgba(0., 0., 0., 1.),
            text: "".to_string(),
            pos,
            num_display: display_info.len(),
            fill_shape: false,
            color_picker: false,
            edit: false,
        }
    }

    pub fn screen(&mut self, ctx: &mut EventCtx, full_screen: Option<screenshots::DisplayInfo>) {
        let a = screenshots::DisplayInfo::all();

        let _display_info = match a {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        let b;
        let display;
        let c;

        if full_screen.is_none() {
            display = screenshots::DisplayInfo::from_point(
                ((self.rect.start_point.unwrap().x - self.pos.x.abs()) * self.scale as f64) as i32,
                ((self.rect.start_point.unwrap().y - self.pos.y.abs()) * self.scale as f64) as i32,
            )
            .unwrap();

            b = screenshots::Screen::new(&display);
            c = b.capture_area(
                (self.rect.start_point.unwrap().x * self.scale as f64
                    - (display.x as f32 * display.scale_factor) as f64
                    + (self.pos.x * self.scale as f64)) as i32,
                (self.rect.start_point.unwrap().y * self.scale as f64
                    - (display.y as f32 * display.scale_factor) as f64
                    + (self.pos.y * self.scale as f64)) as i32,
                (self.rect.size.width as f32 * self.scale) as u32,
                (self.rect.size.height as f32 * self.scale) as u32,
            );
        } else {
            b = screenshots::Screen::new(&(full_screen.unwrap()));
            if self.is_full_screen {
                c = b.capture();
            } else {
                //prendo lo start point x e sommo il pos x poi prendi display x e sottrai/sommi pos e poi quella la sottrai a quella che hai ottenuto prima funziona
                c = b.capture_area(
                    (self.rect.start_point.unwrap().x * self.scale as f64
                        - (full_screen.unwrap().x as f32 * full_screen.unwrap().scale_factor)
                            as f64
                        + (self.pos.x * self.scale as f64)) as i32,
                    (self.rect.start_point.unwrap().y * self.scale as f64
                        - (full_screen.unwrap().y as f32 * full_screen.unwrap().scale_factor)
                            as f64
                        + (self.pos.y * self.scale as f64)) as i32,
                    (self.rect.size.width as f32 * self.scale) as u32,
                    (self.rect.size.height as f32 * self.scale) as u32,
                );
                //self.rect = SelectionRectangle::default();
            }
        }

        let image = match c {
            Err(why) => return println!("qui {}", why),
            Ok(info) => info,
        };

        self.img = ImageBuf::from_raw(
            image.clone().into_raw(),
            druid::piet::ImageFormat::RgbaPremul,
            image.clone().width() as usize,
            image.clone().height() as usize,
        );

        self.tool_window.img = self.img.clone();

        let width = self.img.width() as f64;
        let height = self.img.height() as f64;

        self.tool_window.img_size.width = self.tool_window.width;
        self.tool_window.img_size.height = height / (width / self.tool_window.width);
        if self.tool_window.img_size.height > self.tool_window.height {
            self.tool_window.img_size.height = self.tool_window.height;
            self.tool_window.img_size.width = width / (height / self.tool_window.height);
        }

        self.tool_window.origin = druid::Point::new(
            self.tool_window.center.x - (self.tool_window.img_size.width / 2.),
            self.tool_window.center.y - (self.tool_window.img_size.height / 2.),
        );

        let window = WindowDesc::new(build_ui(self.scale))
            .menu(make_menu)
            .title("Screen grabbing")
            .window_size((1200., 750.))
            .resizable(false)
            .set_position(Point::new(0., 0.))
            .set_window_state(WindowState::Restored);
        ctx.new_window(window);
        self.selection_transparency = 0.4;
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
            self.tool_window.img.width() as u32,
            self.tool_window.img.height() as u32,
            self.tool_window.img.raw_pixels().to_vec(),
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
        //self.rect = SelectionRectangle::default();
    }

    pub fn save_as(&mut self, path: &Path) {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
            self.tool_window.img.width() as u32,
            self.tool_window.img.height() as u32,
            self.tool_window.img.raw_pixels().to_vec(),
        )
        .unwrap();
        image
            .save_with_format(path, image::ImageFormat::Png)
            .expect("Error saving");
        //self.rect = SelectionRectangle::default();
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
pub struct SelectionShape {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
    pub center: Option<Point>,
    pub radii: Option<druid::Vec2>,
    pub filled: bool,
}

impl Default for SelectionShape {
    fn default() -> Self {
        SelectionShape {
            start_point: None, //p1
            end_point: None,   // p4
            center: None,
            radii: None,
            filled: true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Draw {
    Shape {
        shape: SelectionShape,
    },
    Text {
        text: String,
        text_pos: druid::Point,
        font: Font<'static>,
    },
    Free {
        points: Vec<Point>,
    },
    Resize {
        res: SelectionRectangle,
    },
}

#[derive(Clone, Data, Lens, Debug)]
pub struct AnnotationTools {
    pub tool: Tools,
    pub center: druid::Point,
    pub origin: druid::Point,
    pub width: f64,
    pub height: f64,
    pub img_size: Size,
    pub rect_stroke: f64,
    pub rect_transparency: f64,
    pub shape: SelectionShape,
    pub img: ImageBuf,
    pub text: String,
    pub text_pos: Option<druid::Point>,
    pub random_point: Option<Point>,
    #[data(ignore)]
    pub draws: Vec<(Draw, Tools, Color)>,
}

impl Default for AnnotationTools {
    fn default() -> Self {
        AnnotationTools {
            tool: Tools::No,
            center: druid::Point::new(400., 250.),
            origin: druid::Point::new(0., 0.),
            width: 800.,
            height: 500.,
            img_size: Size::ZERO,
            rect_stroke: 0.0,
            rect_transparency: 0.0,
            shape: SelectionShape::default(),
            img: ImageBuf::empty(),
            text: "".to_string(),
            text_pos: None,
            random_point: None,
            draws: Vec::new(),
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

#[derive(Clone, PartialEq, Debug)]
pub struct FreeRect {
    pub p1: imageproc::point::Point<i32>,
    pub p2: imageproc::point::Point<i32>,
    pub p3: imageproc::point::Point<i32>,
    pub p4: imageproc::point::Point<i32>,
}

impl FreeRect {
    pub fn new(a: druid::Point, b: druid::Point, c: druid::Point, d: druid::Point) -> Self {
        let p1 = imageproc::point::Point::new(a.x as i32, a.y as i32);
        let p2 = imageproc::point::Point::new(b.x as i32, b.y as i32);
        let p3 = imageproc::point::Point::new(c.x as i32, c.y as i32);
        let p4 = imageproc::point::Point::new(d.x as i32, d.y as i32);

        return FreeRect {
            p1: p1,
            p2: p2,
            p3: p3,
            p4: p4,
        };
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
    All,
}

//Controller to take screen after the custom shortcut
pub struct Enter {
    pub id_t: TimerToken,
    pub id_t2: TimerToken,
    pub locks: [bool; 5],
    pub do_screen: bool,
    pub witch_screen: u32,
    pub display: Option<screenshots::DisplayInfo>,
    pub hotkey: druid::HotKey,
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
        if self.do_screen {
            match event {
                Event::Timer(id) => {
                    if self.id_t == *id {
                        ctx.window().close();
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        if self.witch_screen == 1 {
                            //1=full screen

                            data.rect.start_point = Some(Point::new(0., 0.));
                            data.rect.end_point = Some(data.size);
                            data.rect.p2 = Some(Point::new(data.size.x, 0.));
                            data.rect.p3 = Some(Point::new(0., data.size.y));
                            let new_win = WindowDesc::new(drag_motion_ui(true))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((data.size.x as f64, data.size.y as f64))
                                .resizable(false)
                                .set_position(data.pos)
                                .set_window_state(WindowState::Restored)
                                .set_always_on_top(true);
                            ctx.new_window(new_win);
                            data.tool_window = AnnotationTools::default();
                        } else {
                            data.rect = SelectionRectangle::default();
                            let current = ctx.window().clone();
                            current.close();
                            let new_win = WindowDesc::new(drag_motion_ui(false))
                                .show_titlebar(false)
                                .transparent(true)
                                .window_size((data.size.x as f64, data.size.y as f64))
                                .resizable(false)
                                .set_position(data.pos);
                            ctx.new_window(new_win);
                            data.tool_window = AnnotationTools::default();
                        }
                    }
                    if self.id_t2 == *id {
                        self.do_screen = false;
                        //ctx.window().close();
                    }
                }
                _ => {}
            }
        } else {
            match event {
                Event::Timer(id) => {
                    if self.id_t == *id {
                        //println!("is full:{:?}  len:{:?}",data.receiver.is_full(),data.receiver.len());
                        if data.receiver.is_full() {
                            self.witch_screen = data.receiver.recv().unwrap();
                            //ctx.window().hide();
                            ctx.window().clone().set_window_state(WindowState::Restored);
                            ctx.window().hide();
                            self.do_screen = true;
                        }
                        self.id_t = ctx.request_timer(Duration::from_millis(100));
                    }
                }
                _ => {
                    self.id_t = ctx.request_timer(Duration::from_millis(100));
                }
            }
        }

        /*
        match event {
            Event::KeyDown(key) => {
                let mut keyboard_event = KeyboardEvent {
                    state: key.state,
                    key: key.key.clone(),
                    code: key.code,
                    location: key.location,
                    modifiers: key.mods.raw(),
                    repeat: key.repeat,
                    is_composing: true,
                };

                if self.hotkey.matches(key){
                    println!("bene");
                }

                match keyboard_event.key {
                    druid::keyboard_types::Key::CapsLock => {
                        self.locks[0] = true;
                    }
                    druid::keyboard_types::Key::FnLock => {
                        self.locks[1] = true;
                    }
                    druid::keyboard_types::Key::NumLock => {
                        self.locks[2] = true;
                    }
                    druid::keyboard_types::Key::ScrollLock => {
                        self.locks[3] = true;
                    }
                    druid::keyboard_types::Key::SymbolLock => {
                        self.locks[4] = true;
                    }
                    _ => {}
                }

                let mut ind = 0;
                for i in self.locks {
                    match ind {
                        0 => {
                            if i {
                                keyboard_event
                                    .modifiers
                                    .insert(druid::keyboard_types::Modifiers::CAPS_LOCK);
                            } else {
                                keyboard_event
                                    .modifiers
                                    .remove(druid::keyboard_types::Modifiers::CAPS_LOCK);
                            }
                        }
                        1 => {
                            if i {
                                keyboard_event
                                    .modifiers
                                    .insert(druid::keyboard_types::Modifiers::FN_LOCK);
                            } else {
                                keyboard_event
                                    .modifiers
                                    .remove(druid::keyboard_types::Modifiers::FN_LOCK);
                            }
                        }
                        2 => {
                            if i {
                                keyboard_event
                                    .modifiers
                                    .insert(druid::keyboard_types::Modifiers::NUM_LOCK);
                            } else {
                                keyboard_event
                                    .modifiers
                                    .remove(druid::keyboard_types::Modifiers::NUM_LOCK);
                            }
                        }
                        3 => {
                            if i {
                                keyboard_event
                                    .modifiers
                                    .insert(druid::keyboard_types::Modifiers::SCROLL_LOCK);
                            } else {
                                keyboard_event
                                    .modifiers
                                    .remove(druid::keyboard_types::Modifiers::SCROLL_LOCK);
                            }
                        }
                        4 => {
                            if i {
                                keyboard_event
                                    .modifiers
                                    .insert(druid::keyboard_types::Modifiers::SYMBOL_LOCK);
                            } else {
                                keyboard_event
                                    .modifiers
                                    .remove(druid::keyboard_types::Modifiers::SYMBOL_LOCK);
                            }
                        }
                        _ => {}
                    }
                    ind = ind + 1;
                }

                let k = char::from_u32(data.key).unwrap();

                if keyboard_event.modifiers
                    == druid::keyboard_types::Modifiers::from_bits(data.mods).unwrap()
                    && (keyboard_event.key == Key::Character(k.to_lowercase().to_string())
                        || keyboard_event.key == Key::Character(k.to_uppercase().to_string()))
                {
                    data.is_full_screen = true;
                    match data.delay {
                        Timer::Zero => self.id_t = ctx.request_timer(Duration::from_millis(100)),
                        _ => {
                            self.id_t =
                                ctx.request_timer(Duration::from_secs(data.delay.set_timer()))
                        }
                    }

                    ctx.window().clone().hide();
                    //self.id_t = ctx.request_timer(Duration::from_millis(100));
                    self.locks = [false; 5];
                }
            }
            Event::Timer(id) => {
                if self.id_t == *id {
                    self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                    self.id_t = TimerToken::next();
                } else if self.id_t2 == *id {
                    data.screen(ctx, self.display);
                    ctx.window().close();
                }
            }
            _ => child.event(ctx, event, data, env),
        }
        */
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

pub struct ShortcutController {}

impl<W: Widget<AppState>> Controller<AppState, W> for ShortcutController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::WindowDisconnected => {
                data.full_mod1.modifier = data.full_mods.0;
                data.full_mod2.modifier = data.full_mods.1;
                data.full_mod3.modifier = data.full_mods.2;
                data.full_k = data.full_key.name().to_string().pop().unwrap().to_string();
                data.area_mod1.modifier = data.area_mods.0;
                data.area_mod2.modifier = data.area_mods.1;
                data.area_mod3.modifier = data.area_mods.2;
                data.area_k = data.area_key.name().to_string().pop().unwrap().to_string();
            }
            _ => {}
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
    pub display: Option<screenshots::DisplayInfo>,
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
                    //USA screenshots::DisplayInfo from point
                    data.from = mouse_down.pos;
                    data.rect.start_point = Some(mouse_button.pos);
                    data.rect.end_point = Some(mouse_button.pos);
                    data.selection_transparency = 0.4;
                    self.display = Some(
                        screenshots::DisplayInfo::from_point(
                            ((data.rect.start_point.unwrap().x - data.pos.x.abs())
                                * data.scale as f64) as i32,
                            ((data.rect.start_point.unwrap().y - data.pos.y.abs())
                                * data.scale as f64) as i32,
                        )
                        .unwrap(),
                    );
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

                    data.rect.end_point = Some(mouse_button.pos);
                    let _mouse_up2 = Point::new(
                        (mouse_button.pos.x - data.pos.x.abs()) * data.scale as f64,
                        (mouse_button.pos.y - data.pos.y.abs()) * data.scale as f64,
                    );
                    let r = druid::Rect::from_points(data.from, mouse_up.pos);
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

                    ctx.window().clone().hide();
                }
                Event::MouseMove(mouse_button) => {
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
                    if !self.display.is_none() {
                        let display = screenshots::DisplayInfo::from_point(
                            ((mouse_move.pos.x - data.pos.x.abs()) * data.scale as f64) as i32,
                            ((mouse_move.pos.y - data.pos.y.abs()) * data.scale as f64) as i32,
                        )
                        .unwrap();
                        if display.id != self.display.unwrap().id {
                            if mouse_move.pos.x - self.display.unwrap().x as f64 > self.display.unwrap().width as f64
                            {
                                if mouse_move.pos.y - self.display.unwrap().y as f64 > self.display.unwrap().height as f64
                                {
                                    data.rect.end_point = Some(Point::new(
                                        self.display.unwrap().width as f64 / data.scale as f64,
                                        self.display.unwrap().height as f64 / data.scale as f64,
                                    ));                                    
                                } else {
                                    data.rect.end_point = Some(Point::new(
                                        self.display.unwrap().width as f64 / data.scale as f64,
                                        mouse_move.pos.y,
                                    ));
                                }
                            }
                        } else {
                            data.rect.end_point = Some(mouse_move.pos);
                        }
                    } else {
                        data.rect.end_point = Some(mouse_move.pos);
                    }
                }
                Event::Timer(id) => {
                    if self.id_t == *id {
                        ctx.window().clone().show();
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        self.id_t = TimerToken::next();
                    } else if self.id_t2 == *id {
                        data.screen(ctx, self.display);
                        //data.resize = true;
                        ctx.window().close();
                    }
                }
                _ => child.event(ctx, event, data, env),
            }
        } else {
            if data.num_display > 1 {
                data.cursor.typ = Cursor::Pointer;
                ctx.set_cursor(&data.cursor.typ);
            }
            match event {
                Event::MouseUp(mouse_button) => {
                    self.display = Some(
                        screenshots::DisplayInfo::from_point(
                            ((mouse_button.pos.x - data.pos.x.abs()) * data.scale as f64) as i32,
                            ((mouse_button.pos.y - data.pos.y.abs()) * data.scale as f64) as i32,
                        )
                        .unwrap(),
                    );

                    data.is_full_screen = true;
                    match data.delay {
                        Timer::Zero => self.id_t = ctx.request_timer(Duration::from_millis(100)),
                        _ => {
                            self.id_t =
                                ctx.request_timer(Duration::from_secs(data.delay.set_timer()))
                        }
                    }

                    ctx.window().clone().hide();
                }
                Event::Timer(id) => {
                    if self.id_t == *id {
                        //ctx.window().clone().set_window_state(WindowState::Restored);
                        ctx.window().clone().show();
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        self.id_t = TimerToken::next();
                    } else if self.id_t2 == *id {
                        data.screen(ctx, self.display);
                        data.cursor.typ = Cursor::Arrow;
                        ctx.set_cursor(&data.cursor.typ);
                        ctx.window().close();
                    }
                }
                _ => {
                    if data.num_display == 1 {
                        data.is_full_screen = true;
                        match data.delay {
                            Timer::Zero => {
                                self.id_t = ctx.request_timer(Duration::from_millis(200))
                            }
                            _ => {
                                self.id_t =
                                    ctx.request_timer(Duration::from_secs(data.delay.set_timer()))
                            }
                        }

                        ctx.window().clone().hide();
                    }
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

pub struct ResizeController {
    pub points: Vec<Point>,
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
        match data.tool_window.tool {
            Tools::Resize => {
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
                            if mouse_button.pos.y > rect.min_y()
                                && mouse_button.pos.y < rect.max_y()
                            {
                                // cambia cursore -> sinistra
                                data.cursor.typ = Cursor::ResizeLeftRight;
                                data.cursor.over = Some(Direction::Left);
                            }
                        } else if (mouse_button.pos.x - rect.max_x()).abs() <= 10. {
                            if mouse_button.pos.y > rect.min_y()
                                && mouse_button.pos.y < rect.max_y()
                            {
                                // cambia cursore -> destra
                                data.cursor.typ = Cursor::ResizeLeftRight;
                                data.cursor.over = Some(Direction::Right);
                            }
                        } else if (mouse_button.pos.y - rect.max_y()).abs() <= 10. {
                            if mouse_button.pos.x > rect.min_x()
                                && mouse_button.pos.x < rect.max_x()
                            {
                                // cambia cursore -> verticale
                                data.cursor.typ = Cursor::ResizeUpDown;
                                data.cursor.over = Some(Direction::Down);
                            }
                        } else if (mouse_button.pos.y - rect.min_y()).abs() <= 10. {
                            if mouse_button.pos.x > rect.min_x()
                                && mouse_button.pos.x < rect.max_x()
                            {
                                // cambia cursore -> verticale
                                data.cursor.typ = Cursor::ResizeUpDown;
                                data.cursor.over = Some(Direction::Up);
                            }
                        } else if (mouse_button.pos.x
                            <= (rect.min_x() + (rect.max_x() - rect.min_x()) * 0.6))
                            && (mouse_button.pos.x
                                >= (rect.min_x() + (rect.max_x() - rect.min_x()) * 0.4))
                            && (mouse_button.pos.y
                                <= (rect.min_y() + (rect.max_y() - rect.min_y()) * 0.6))
                            && (mouse_button.pos.y
                                >= (rect.min_y() + (rect.max_y() - rect.min_y()) * 0.4))
                        {
                            data.cursor.typ = Cursor::Crosshair;
                            data.cursor.over = Some(Direction::All);
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
                                if mouse_button.pos.y < data.rect.p3.unwrap().y - 10.
                                    && mouse_button.pos.y
                                        > (data.tool_window.center.y
                                            - data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.start_point.replace(Point::new(
                                        data.rect.start_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.p2.replace(Point::new(
                                        data.rect.p2.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::Down) => {
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10.
                                    && mouse_button.pos.y
                                        < (data.tool_window.center.y
                                            + data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.end_point.replace(Point::new(
                                        data.rect.end_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.p3.replace(Point::new(
                                        data.rect.p3.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::Left) => {
                                if mouse_button.pos.x < data.rect.p2.unwrap().x - 10.
                                    && mouse_button.pos.x
                                        > (data.tool_window.center.x
                                            - data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                    data.rect.p3.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p3.unwrap().y,
                                    ));
                                }
                            }
                            Some(Direction::Right) => {
                                if mouse_button.pos.x > data.rect.start_point.unwrap().x + 10.
                                    && mouse_button.pos.x
                                        < (data.tool_window.center.x
                                            + data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                    data.rect.p2.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p2.unwrap().y,
                                    ));
                                }
                            }
                            Some(Direction::UpLeft) => {
                                if mouse_button.pos.x < data.rect.end_point.unwrap().x - 10.
                                    && mouse_button.pos.x
                                        > (data.tool_window.center.x
                                            - data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                    data.rect.p3.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p3.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y < data.rect.end_point.unwrap().y - 10.
                                    && mouse_button.pos.y
                                        > (data.tool_window.center.y
                                            - data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.start_point.replace(Point::new(
                                        data.rect.start_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.p2.replace(Point::new(
                                        data.rect.p2.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::UpRight) => {
                                if mouse_button.pos.x > data.rect.p3.unwrap().x + 10.
                                    && mouse_button.pos.x
                                        < (data.tool_window.center.x
                                            + data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.p2.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p2.unwrap().y,
                                    ));
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y < data.rect.end_point.unwrap().y - 10.
                                    && mouse_button.pos.y
                                        > (data.tool_window.center.y
                                            - data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.p2.replace(Point::new(
                                        data.rect.p2.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.start_point.replace(Point::new(
                                        data.rect.start_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::DownLeft) => {
                                if mouse_button.pos.x < data.rect.end_point.unwrap().x - 10.
                                    && mouse_button.pos.x
                                        > (data.tool_window.center.x
                                            - data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.p3.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p3.unwrap().y,
                                    ));
                                    data.rect.start_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.start_point.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10.
                                    && mouse_button.pos.y
                                        < (data.tool_window.center.y
                                            + data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.p3.replace(Point::new(
                                        data.rect.p3.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.end_point.replace(Point::new(
                                        data.rect.end_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::DownRight) => {
                                if mouse_button.pos.x > data.rect.p3.unwrap().x + 10.
                                    && mouse_button.pos.x
                                        < (data.tool_window.center.x
                                            + data.tool_window.img_size.width / 2.)
                                {
                                    data.rect.end_point.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.end_point.unwrap().y,
                                    ));
                                    data.rect.p2.replace(Point::new(
                                        mouse_button.pos.x,
                                        data.rect.p2.unwrap().y,
                                    ));
                                }
                                if mouse_button.pos.y > data.rect.start_point.unwrap().y + 10.
                                    && mouse_button.pos.y
                                        < (data.tool_window.center.y
                                            + data.tool_window.img_size.height / 2.)
                                {
                                    data.rect.end_point.replace(Point::new(
                                        data.rect.end_point.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                    data.rect.p3.replace(Point::new(
                                        data.rect.p3.unwrap().x,
                                        mouse_button.pos.y,
                                    ));
                                }
                            }
                            Some(Direction::All) => {
                                if mouse_button.pos.x
                                    > (data.tool_window.origin.x + data.rect.size.width / 2.)
                                    && (mouse_button.pos.x
                                        < data.tool_window.origin.x
                                            + data.tool_window.img_size.width
                                            - data.rect.size.width / 2.)
                                    && mouse_button.pos.y
                                        > (data.tool_window.origin.y + data.rect.size.height / 2.)
                                    && (mouse_button.pos.y
                                        < data.tool_window.origin.y
                                            + data.tool_window.img_size.height
                                            - data.rect.size.height / 2.)
                                {
                                    let rect2 = druid::Rect::from_center_size(
                                        mouse_button.pos,
                                        data.rect.size,
                                    );

                                    data.rect.start_point.replace(rect2.origin());
                                    data.rect
                                        .end_point
                                        .replace(Point::new(rect2.max_x(), rect2.max_y()));
                                    data.rect.p2 = Some(Point::new(0., rect2.max_y()));
                                    data.rect.p3 = Some(Point::new(rect2.max_x(), 0.));
                                }
                            }
                            None => return, // non è sopra il bordo
                        }
                    }
                }
            }
            Tools::Ellipse => match event {
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
                    data.tool_window.shape.start_point = Some(mouse_down.pos);
                    data.tool_window.shape.end_point = Some(mouse_down.pos);

                    data.selection_transparency = 1.;
                }
                Event::MouseMove(mouse_button) => {
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

                    data.tool_window.shape.end_point = Some(mouse_move.pos);
                    if let (Some(start), Some(end)) = (
                        data.tool_window.shape.start_point,
                        data.tool_window.shape.end_point,
                    ) {
                        let radius1 = (start.x - end.x) / 2.;
                        let radius2 = (start.y - end.y) / 2.;
                        let c1 = end.x + radius1;
                        let c2 = end.y + radius2;
                        data.tool_window.shape.center = Some(druid::Point::new(c1, c2));
                        data.tool_window.shape.radii =
                            Some(druid::Vec2::new(radius1.abs(), radius2.abs()));
                    }
                }
                Event::MouseUp(mouse_button) => {
                    let _mouse_up = MouseEvent {
                        pos: mouse_button.pos,
                        window_pos: mouse_button.window_pos,
                        buttons: mouse_button.buttons,
                        mods: mouse_button.mods,
                        count: mouse_button.count,
                        focus: mouse_button.focus,
                        button: mouse_button.button,
                        wheel_delta: mouse_button.wheel_delta,
                    };

                    data.selection_transparency = 0.;

                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                        data.tool_window.img.width() as u32,
                        data.tool_window.img.height() as u32,
                        data.tool_window.img.raw_pixels().to_vec(),
                    )
                    .unwrap();

                    let color = data.color.as_rgba8();

                    if !data.fill_shape {
                        for i in -50..50 {
                            imageproc::drawing::draw_hollow_ellipse_mut(
                                &mut image,
                                (
                                    ((data.tool_window.shape.center.unwrap().x
                                        - data.tool_window.origin.x)
                                        * (data.tool_window.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        .round() as i32,
                                    ((data.tool_window.shape.center.unwrap().y
                                        - data.tool_window.origin.y)
                                        * (data.tool_window.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        .round() as i32,
                                ),
                                ((data.tool_window.shape.radii.unwrap().x - i as f64 / 20.)
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as i32,
                                ((data.tool_window.shape.radii.unwrap().y - i as f64 / 20.)
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as i32,
                                Rgba([color.0, color.1, color.2, 255]),
                            );
                        }
                        data.tool_window.shape.filled = false;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Ellipse,
                            data.color.clone(),
                        ));
                    } else {
                        imageproc::drawing::draw_filled_ellipse_mut(
                            &mut image,
                            (
                                ((data.tool_window.shape.center.unwrap().x
                                    - data.tool_window.origin.x)
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as i32,
                                ((data.tool_window.shape.center.unwrap().y
                                    - data.tool_window.origin.y)
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as i32,
                            ),
                            (data.tool_window.shape.radii.unwrap().x
                                * (data.tool_window.img.width() as f64
                                    / data.tool_window.img_size.width))
                                as i32,
                            (data.tool_window.shape.radii.unwrap().y
                                * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height))
                                as i32,
                            Rgba([color.0, color.1, color.2, 255]),
                        );
                        data.tool_window.shape.filled = true;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Ellipse,
                            data.color.clone(),
                        ));
                    }

                    data.tool_window.img = ImageBuf::from_raw(
                        image.clone().into_raw(),
                        druid::piet::ImageFormat::RgbaPremul,
                        image.clone().width() as usize,
                        image.clone().height() as usize,
                    );

                    data.tool_window.shape.start_point = None;
                    data.tool_window.shape.end_point = None;
                    data.tool_window.shape.center = None;
                    data.tool_window.shape.radii = None;
                }
                _ => {}
            },
            Tools::Rectangle => match event {
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
                    data.tool_window.shape.start_point = Some(mouse_down.pos);
                    data.tool_window.shape.end_point = Some(mouse_down.pos);

                    data.selection_transparency = 1.;
                }
                Event::MouseMove(mouse_button) => {
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

                    data.tool_window.shape.end_point = Some(mouse_move.pos);
                }
                Event::MouseUp(mouse_button) => {
                    let _mouse_up = MouseEvent {
                        pos: mouse_button.pos,
                        window_pos: mouse_button.window_pos,
                        buttons: mouse_button.buttons,
                        mods: mouse_button.mods,
                        count: mouse_button.count,
                        focus: mouse_button.focus,
                        button: mouse_button.button,
                        wheel_delta: mouse_button.wheel_delta,
                    };

                    data.selection_transparency = 0.;

                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                        data.tool_window.img.width() as u32,
                        data.tool_window.img.height() as u32,
                        data.tool_window.img.clone().raw_pixels().to_vec(),
                    )
                    .unwrap();

                    let color = data.color.as_rgba8();

                    if !data.fill_shape {
                        for i in -20..20 {
                            imageproc::drawing::draw_hollow_rect_mut(
                                &mut image,
                                imageproc::rect::Rect::at(
                                    ((data
                                        .tool_window
                                        .shape
                                        .start_point
                                        .unwrap()
                                        .x
                                        .min(data.tool_window.shape.end_point.unwrap().x)
                                        + i as f64 / 10.
                                        - data.tool_window.origin.x)
                                        * (data.tool_window.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        as i32,
                                    ((data
                                        .tool_window
                                        .shape
                                        .start_point
                                        .unwrap()
                                        .y
                                        .min(data.tool_window.shape.end_point.unwrap().y)
                                        + i as f64 / 10.
                                        - data.tool_window.origin.y)
                                        * (data.tool_window.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        as i32,
                                )
                                .of_size(
                                    ((((data.tool_window.shape.start_point.unwrap().x
                                        - data.tool_window.shape.end_point.unwrap().x)
                                        .abs()
                                        - 2. * i as f64 / 10.)
                                        * (data.tool_window.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        as u32)
                                        .max(1),
                                    ((((data.tool_window.shape.start_point.unwrap().y
                                        - data.tool_window.shape.end_point.unwrap().y)
                                        .abs()
                                        - 2. * i as f64 / 10.)
                                        * (data.tool_window.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        as u32)
                                        .max(1),
                                ),
                                Rgba([color.0, color.1, color.2, 255]),
                            );
                        }
                        data.tool_window.shape.filled = false;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Rectangle,
                            data.color.clone(),
                        ));
                    } else {
                        imageproc::drawing::draw_filled_rect_mut(
                            &mut image,
                            imageproc::rect::Rect::at(
                                ((data
                                    .tool_window
                                    .shape
                                    .start_point
                                    .unwrap()
                                    .x
                                    .min(data.tool_window.shape.end_point.unwrap().x)
                                    - data.tool_window.origin.x)
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as i32,
                                ((data
                                    .tool_window
                                    .shape
                                    .start_point
                                    .unwrap()
                                    .y
                                    .min(data.tool_window.shape.end_point.unwrap().y)
                                    - data.tool_window.origin.y)
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as i32,
                            )
                            .of_size(
                                (((data.tool_window.shape.start_point.unwrap().x
                                    - data.tool_window.shape.end_point.unwrap().x)
                                    .abs())
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as u32,
                                (((data.tool_window.shape.start_point.unwrap().y
                                    - data.tool_window.shape.end_point.unwrap().y)
                                    .abs())
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as u32,
                            ),
                            Rgba([color.0, color.1, color.2, 255]),
                        );
                        data.tool_window.shape.filled = true;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Rectangle,
                            data.color.clone(),
                        ));
                    }

                    data.tool_window.img = ImageBuf::from_raw(
                        image.clone().into_raw(),
                        druid::piet::ImageFormat::RgbaPremul,
                        image.clone().width() as usize,
                        image.clone().height() as usize,
                    );

                    data.tool_window.shape.start_point = None;
                    data.tool_window.shape.end_point = None;
                    data.tool_window.shape.center = None;
                    data.tool_window.shape.radii = None;
                }
                _ => {}
            },
            Tools::Arrow => match event {
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
                    data.tool_window.shape.start_point = Some(mouse_down.pos);
                    data.tool_window.shape.end_point = Some(mouse_down.pos);

                    data.tool_window.rect_transparency = 1.;
                }
                Event::MouseMove(mouse_button) => {
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

                    data.tool_window.shape.end_point = Some(mouse_move.pos);
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

                    data.tool_window.rect_transparency = 0.;
                    data.tool_window.shape.end_point = Some(mouse_up.pos);

                    let end = data.tool_window.shape.end_point.unwrap();
                    let start = data.tool_window.shape.start_point.unwrap();

                    if end == start {
                        //data.tool_window.img = data.img.clone();
                        data.tool_window.tool = Tools::No;
                        child.event(ctx, event, data, env);
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
                            data.tool_window.shape.start_point.unwrap(),
                            data.tool_window.shape.start_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
                        );
                        let mut line1 = FreeRect::new(
                            end1,
                            end1,
                            data.tool_window.shape.end_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
                        );
                        let mut line2 = FreeRect::new(
                            end2,
                            end2,
                            data.tool_window.shape.end_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
                        );

                        let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                            data.tool_window.img.width() as u32,
                            data.tool_window.img.height() as u32,
                            data.tool_window.img.clone().raw_pixels().to_vec(),
                        )
                        .unwrap();

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
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        body.p1.y = ((body.p1.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        body.p2.x = ((body.p2.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        body.p2.y = ((body.p2.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        body.p3.x = ((body.p3.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        body.p3.y = ((body.p3.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        body.p4.x = ((body.p4.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        body.p4.y = ((body.p4.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;

                        line1.p1.x = ((line1.p1.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line1.p1.y = ((line1.p1.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line1.p2.x = ((line1.p2.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line1.p2.y = ((line1.p2.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line1.p3.x = ((line1.p3.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line1.p3.y = ((line1.p3.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line1.p4.x = ((line1.p4.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line1.p4.y = ((line1.p4.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;

                        line2.p1.x = ((line2.p1.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line2.p1.y = ((line2.p1.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line2.p2.x = ((line2.p2.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line2.p2.y = ((line2.p2.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line2.p3.x = ((line2.p3.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line2.p3.y = ((line2.p3.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line2.p4.x = ((line2.p4.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line2.p4.y = ((line2.p4.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;

                        let color = data.color.as_rgba8();

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

                        data.tool_window.img = ImageBuf::from_raw(
                            image.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            image.clone().width() as usize,
                            image.clone().height() as usize,
                        );

                        data.tool_window.shape.filled = true;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Arrow,
                            data.color.clone(),
                        ));

                        data.tool_window.shape.start_point = None;
                        data.tool_window.shape.end_point = None;

                        data.tool_window.shape.start_point = None;
                        data.tool_window.shape.end_point = None;
                        data.tool_window.shape.center = None;
                        data.tool_window.shape.radii = None;
                    }
                }
                _ => {}
            },
            Tools::Text => match event {
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
                    data.color = data.color.with_alpha(1.);
                    data.tool_window.text_pos = Some(mouse_down.pos);
                }
                _ => {}
            },
            Tools::Highlight => {
                ctx.set_cursor(&data.cursor.typ);
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

                        data.color = data.color.with_alpha(0.4);
                        data.tool_window.shape.start_point = Some(mouse_down.pos);
                    }
                    Event::MouseMove(mouse_button) => {
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

                        data.tool_window.shape.end_point = Some(mouse_move.pos);
                    }
                    Event::MouseUp(_mouse_button) => {
                        let mut image = ImageBuffer::new(
                            data.tool_window.img.width() as u32,
                            data.tool_window.img.height() as u32,
                        );

                        let mut line = FreeRect::new(
                            data.tool_window.shape.start_point.unwrap(),
                            data.tool_window.shape.start_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
                            data.tool_window.shape.end_point.unwrap(),
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
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line.p1.y = ((line.p1.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line.p2.x = ((line.p2.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line.p2.y = ((line.p2.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line.p3.x = ((line.p3.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line.p3.y = ((line.p3.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;
                        line.p4.x = ((line.p4.x as f64 - data.tool_window.origin.x)
                            * (data.tool_window.img.width() as f64
                                / data.tool_window.img_size.width))
                            as i32;
                        line.p4.y = ((line.p4.y as f64 - data.tool_window.origin.y)
                            * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height))
                            as i32;

                        let color = data.color.as_rgba8();
                        let prova = imageproc::drawing::draw_polygon(
                            &mut image,
                            &[line.p1, line.p2, line.p3, line.p4],
                            Rgba([color.0, color.1, color.2, color.3]),
                        );

                        let mut bottom: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                            data.tool_window.img.width() as u32,
                            data.tool_window.img.height() as u32,
                            data.tool_window.img.clone().raw_pixels().to_vec(),
                        )
                        .unwrap();
                        image::imageops::overlay(&mut bottom, &prova, 0, 0);

                        data.tool_window.img = ImageBuf::from_raw(
                            bottom.clone().into_raw(),
                            druid::piet::ImageFormat::RgbaPremul,
                            bottom.clone().width() as usize,
                            bottom.clone().height() as usize,
                        );

                        data.tool_window.shape.filled = true;
                        data.tool_window.draws.push((
                            Draw::Shape {
                                shape: data.tool_window.shape.clone(),
                            },
                            Tools::Highlight,
                            data.color.clone(),
                        ));

                        data.tool_window.shape.start_point = None;
                        data.tool_window.shape.end_point = None;
                        data.color = data.color.with_alpha(1.);
                    }
                    _ => {}
                }
            }
            Tools::Random => match event {
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

                    if mouse_down.pos.x > data.tool_window.origin.x
                        && mouse_down.pos.y > data.tool_window.origin.y
                        && mouse_down.pos.x < data.tool_window.origin.x + data.tool_window.width
                        && mouse_down.pos.y < data.tool_window.height
                    {
                        self.points.push(mouse_down.pos);
                        data.color = data.color.with_alpha(1.);
                    }
                }
                Event::MouseMove(mouse_button) => {
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

                    if mouse_move.pos.x > data.tool_window.origin.x
                        && mouse_move.pos.y > data.tool_window.origin.y
                        && mouse_move.pos.x < data.tool_window.origin.x + data.tool_window.width
                        && mouse_move.pos.y < data.tool_window.height
                        && data.color.as_rgba8().3 == 255
                    {
                        self.points.push(mouse_move.pos);
                    } else {
                        if !self.points.is_empty() {
                            self.points = Vec::new();
                        }
                    }
                    data.tool_window.random_point = Some(mouse_move.pos);
                }
                Event::MouseUp(mouse_button) => {
                    data.color = data.color.with_alpha(1.);
                    let color = data.color.as_rgba8();

                    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_vec(
                        data.tool_window.img.width() as u32,
                        data.tool_window.img.height() as u32,
                        data.tool_window.img.clone().raw_pixels().to_vec(),
                    )
                    .unwrap();
                    if !self.points.is_empty() && self.points.len() >= 1 {
                        for i in 0..self.points.len() - 1 {
                            let mut line = FreeRect::new(
                                self.points[i],
                                self.points[i],
                                self.points[i + 1],
                                self.points[i + 1],
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
                                * (data.tool_window.img.width() as f64
                                    / data.tool_window.img_size.width))
                                as i32;
                            line.p1.y = ((line.p1.y as f64 - data.tool_window.origin.y)
                                * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height))
                                as i32;
                            line.p2.x = ((line.p2.x as f64 - data.tool_window.origin.x)
                                * (data.tool_window.img.width() as f64
                                    / data.tool_window.img_size.width))
                                as i32;
                            line.p2.y = ((line.p2.y as f64 - data.tool_window.origin.y)
                                * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height))
                                as i32;
                            line.p3.x = ((line.p3.x as f64 - data.tool_window.origin.x)
                                * (data.tool_window.img.width() as f64
                                    / data.tool_window.img_size.width))
                                as i32;
                            line.p3.y = ((line.p3.y as f64 - data.tool_window.origin.y)
                                * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height))
                                as i32;
                            line.p4.x = ((line.p4.x as f64 - data.tool_window.origin.x)
                                * (data.tool_window.img.width() as f64
                                    / data.tool_window.img_size.width))
                                as i32;
                            line.p4.y = ((line.p4.y as f64 - data.tool_window.origin.y)
                                * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height))
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
                                    ((self.points[i].x - data.tool_window.origin.x)
                                        * (data.tool_window.img.width() as f64
                                            / data.tool_window.img_size.width))
                                        as i32,
                                    ((self.points[i].y - data.tool_window.origin.y)
                                        * (data.tool_window.img.height() as f64
                                            / data.tool_window.img_size.height))
                                        as i32,
                                ),
                                2 * (data.tool_window.img.height() as f64
                                    / data.tool_window.img_size.height)
                                    as i32,
                                Rgba([color.0, color.1, color.2, color.3]),
                            );
                        }

                        imageproc::drawing::draw_filled_circle_mut(
                            &mut image,
                            (
                                ((self.points.last().unwrap().x - data.tool_window.origin.x)
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as i32,
                                ((self.points.last().unwrap().y - data.tool_window.origin.y)
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as i32,
                            ),
                            3 * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height)
                                as i32,
                            Rgba([color.0, color.1, color.2, color.3]),
                        );
                    } else {
                        imageproc::drawing::draw_filled_circle_mut(
                            &mut image,
                            (
                                ((mouse_button.pos.x - data.tool_window.origin.x)
                                    * (data.tool_window.img.width() as f64
                                        / data.tool_window.img_size.width))
                                    as i32,
                                ((mouse_button.pos.y - data.tool_window.origin.y)
                                    * (data.tool_window.img.height() as f64
                                        / data.tool_window.img_size.height))
                                    as i32,
                            ),
                            3 * (data.tool_window.img.height() as f64
                                / data.tool_window.img_size.height)
                                as i32,
                            Rgba([color.0, color.1, color.2, color.3]),
                        );
                    }

                    data.tool_window.img = ImageBuf::from_raw(
                        image.clone().into_raw(),
                        druid::piet::ImageFormat::RgbaPremul,
                        image.clone().width() as usize,
                        image.clone().height() as usize,
                    );

                    data.tool_window.draws.push((
                        Draw::Free {
                            points: self.points.clone(),
                        },
                        Tools::Random,
                        data.color.clone(),
                    ));

                    self.points = Vec::new();
                    data.color = data.color.with_alpha(0.);
                }
                _ => {}
            },
            _ => {}
        }

        if !ctx.is_hot() {
            data.cursor.over = None;
            data.cursor.down = false;
        }
        if ctx.is_focused() && data.tool_window.tool != Tools::Text {
            ctx.resign_focus();
            //ctx.focus_prev();
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
                .window_size((600.0, 200.0))
                .resizable(false);
            ctx.new_window(new_win);
        }
        Handled::No
    }
}

fn create_listener(receiver: Receiver<(Hotkey, u32)>, sender: CrossSender<u32>) {
    let mut tasti1 = livesplit_hotkey::Hotkey {
        modifiers: livesplit_hotkey::Modifiers::ALT,
        key_code: livesplit_hotkey::KeyCode::KeyS,
    };
    let mut tasti2 = livesplit_hotkey::Hotkey {
        modifiers: livesplit_hotkey::Modifiers::ALT,
        key_code: livesplit_hotkey::KeyCode::KeyG,
    };
    let mut tasti;

    let hk = livesplit_hotkey::Hook::new().unwrap();

    let mut n_s;

    loop {
        //let keys=tasti.clone();
        let send = sender.clone();

        let _result = hk.register(tasti1, move || {
            send.send(1).expect("Error shortcut");
            //println!("send len:{:?}",send.len());
        });

        let send = sender.clone();

        let _result = hk.register(tasti2, move || {
            send.send(2).expect("Error shortcut");
            //println!("send len:{:?}",send.len());
        });

        let hotkeys = receiver.recv();

        match hotkeys {
            Ok(data) => (tasti, n_s) = data,
            Err(_err) => {
                break;
            }
        }

        if n_s == 1 {
            hk.unregister(tasti1).expect("Error shortcut");
            tasti1 = tasti;
            let send = sender.clone();
            let _result = hk.register(tasti1, move || {
                send.send(1).expect("Error shortcut");
                //println!("send len:{:?}",send.len());
            });
        } else {
            hk.unregister(tasti2).expect("Error shortcut");
            tasti2 = tasti;
            let send = sender.clone();
            let _result = hk.register(tasti2, move || {
                send.send(2).expect("Error shortcut");
                //println!("send len:{:?}",send.len());
            });
        }
    }
}

/*
fn create_listener(receiver:Receiver<(global_hotkey::hotkey::HotKey,u32)>,sender:CrossSender<u32>){

    let mut tasti;

    let mut n_s=0;

    let manager = GlobalHotKeyManager::new().unwrap();
    let (mut tasti1,ind1)=receiver.recv().unwrap();
    let (mut tasti2,ind1)=receiver.recv().unwrap();

    let result=manager.register(tasti1);

    match result{
        Ok(ok)=>{},
        Err(err)=>{println!("cazzo")}
    }
    manager.register(tasti2).unwrap();

    println!("{:?}",tasti1);
    let rec = GlobalHotKeyEvent::receiver();
    let send=sender.clone();

    std::thread::spawn(|| loop {
        if let Ok(event) = rec.try_recv() {
            let id=event.id;
            //send.send(id);

            println!("try event: {event:?}");
        }
        println!("{:?}",rec.len());
        //println!("funziona");
        std::thread::sleep(Duration::from_millis(100));
    });

    loop{
        //let keys=tasti.clone();

        println!("listener creato");

        let hotkeys=receiver.recv();

        match hotkeys {
            Ok(data)=>{(tasti,n_s)=data},
            Err(err)=>{ break;}
        }

        if n_s==1{
            manager.unregister(tasti1);
            tasti1=tasti;
            let result=manager.register(tasti1);
        }else {
            manager.unregister(tasti2);
            tasti2=tasti;
            let result=manager.register(tasti2);
        }

        println!("create");

    }
}
*/
