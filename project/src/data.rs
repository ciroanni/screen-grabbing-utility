use crate::ui::*;
use druid::{
    commands, widget::Controller, AppDelegate, Command, Cursor, Data, DelegateCtx, Env, Event,
    EventCtx, Handled, ImageBuf, Lens, MouseEvent, Point, Target, TimerToken, Widget, WindowDesc,
    WindowState, LocalizedString, Selector, 
};
use druid_shell::keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use image::{ImageBuffer, Rgba};
use std::path::Path;
use std::time::Duration;

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

#[derive(Clone, Data, Lens)]
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
    pub selection_end: bool, //true --> end of area selection
    pub selection_transparency: f64,
    pub img: ImageBuf,
    pub cursor: CursorData,
    pub path: String,
    pub delay: Duration,
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
            selection_end: false,
            selection_transparency: 0.4,
            img,
            cursor: CursorData::new(Cursor::Arrow, None, false),
            path: ".".to_string(),
            delay: Duration::from_secs(0),
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
        if self.rect.is_none() {
            c = b.capture_area(0, 0, (self.size.x) as u32, (self.size.y) as u32);
        } else {
            let origin = druid::Rect::from_points(
                self.rect.start_point.unwrap(),
                self.rect.end_point.unwrap(),
            )
            .origin();
            c = b.capture_area(
                (origin.x as f32 * self.scale) as i32,
                (origin.y as f32 * self.scale) as i32,
                (self.size.x as f32) as u32,
                (self.size.y as f32) as u32,
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

        let window = WindowDesc::new(show_screen_ui(self.img.clone()))
            .menu(make_menu)
            .title("Shortcut")
            .window_size((1000., 1000.));
        ctx.new_window(window);

        //*self = AppState::new(self.scale, self.img.clone());
        ctx.window().close();
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
}

impl Default for SelectionRectangle {
    fn default() -> Self {
        SelectionRectangle {
            start_point: None, //p1
            end_point: None, // p4
            p2: None,
            p3: None,
        }
    }
}
/*
start = p1 .____. p2
           |    |
        p3 .____. p4 = end

 */
impl SelectionRectangle {
    pub fn is_none(&self) -> bool {
        if self.start_point.is_none() && self.end_point.is_none() {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Data, PartialEq, Lens, Debug)]
pub struct CursorData{
    typ: Cursor,
    over: Option<Direction>,
    down: bool
}

impl CursorData{
    pub fn new(typ: Cursor, over: Option<Direction>, down: bool) -> Self {
        Self{
            typ,
            over,
            down,
        }
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
pub struct Enter;

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

            ShortcutMatcher::from_event(keyboard_event).shortcut(
                Modifiers::from_bits(data.mods).expect("Not a modifier"),
                Key::Character(char::from_u32(data.key).expect("Not a char").to_string()),
                || data.screen(ctx),
            );
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
        if !self.flag {
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
                    data.size.x = ((data.from.x - mouse_up.pos.x).abs() as f32 * data.scale) as f64;
                    data.size.y = ((data.from.y - mouse_up.pos.y).abs() as f32 * data.scale) as f64;
                    //data.rect.end_point = Some(mouse_button.pos);
                    let r = druid::Rect::from_points(
                        data.from,
                        mouse_button.pos,
                    );
                    
                    // aggiusto i punti
                    data.rect.start_point = Some(r.origin());
                    data.rect.p2 = Some(Point::new(r.max_x(), r.min_y()));
                    data.rect.p3 = Some(Point::new(r.min_x(), r.max_y()));
                    data.rect.end_point = Some(Point::new(r.max_x(), r.max_y()));
        
                    data.selection_transparency = 0.0;
                    data.selection_end = true;

                    if data.delay <= Duration::from_millis(100) {
                        self.id_t = ctx.request_timer(Duration::from_millis(100));
                    } else {
                        self.id_t = ctx.request_timer(data.delay);
                    }

                    ctx.window()
                        .clone()
                        .set_window_state(WindowState::Minimized);
                }
                Event::MouseMove(mouse_button) => {
                    if !data.rect.start_point.is_none() {
                        data.rect.end_point = Some(mouse_button.pos);
                    }
                }
                Event::Timer(id) => {
                    if self.id_t == *id {
                        ctx.window().clone().set_window_state(WindowState::Restored);
                        self.id_t2 = ctx.request_timer(Duration::from_millis(100));
                        self.id_t = TimerToken::next();
                    } else if self.id_t2 == *id {
                        data.screen(ctx);
                        data.selection_end = false;
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
                        data.selection_end = false;
                        ctx.window().close();
                    }
                }
                _ => {
                    data.selection_end = true;
                    if data.delay <= Duration::from_millis(100) {
                        self.id_t = ctx.request_timer(Duration::from_millis(100));
                    } else {
                        self.id_t = ctx.request_timer(data.delay);
                    }

                    ctx.window()
                        .clone()
                        .set_window_state(WindowState::Minimized)
                }
            }
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

//Controller to resize screening area
pub struct ResizeController;

impl<W: Widget<AppState>> Controller<AppState, W> for ResizeController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        if let Event::MouseDown(mouse_button) = event {
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

            match data.cursor.over {
                Some(_) => {
                    data.cursor.down = true; // è sopra il bordo, sto premendo
                }
                None => return, // non è sopra il bordo
            }

            // data.from = mouse_down.pos;
            // data.rect.start_point = Some(mouse_button.pos);
            // data.rect.end_point = None;
        } else if let Event::MouseUp(mouse_button) = event {
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
            
            data.cursor.down = false; // è sopra il bordo, sto premendo
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
                if (mouse_button.pos.x - rect.min_x()).abs() <= 10. && (mouse_button.pos.y - rect.min_y()).abs() <= 10. {
                    data.cursor.typ = Cursor::Crosshair;
                    data.cursor.over = Some(Direction::UpLeft);
                }
                else if(mouse_button.pos.x - rect.max_x()).abs() <= 10. && (mouse_button.pos.y - rect.max_y()).abs() <= 10.{
                    data.cursor.typ = Cursor::Crosshair;
                    data.cursor.over = Some(Direction::DownRight);
                }
                // cambia cursore -> antidiagonale
                else if (mouse_button.pos.x - rect.min_x()).abs() <= 10. && (mouse_button.pos.y - rect.max_y()).abs() <= 10. {
                    data.cursor.typ = Cursor::NotAllowed;
                    data.cursor.over = Some(Direction::DownLeft);
                }
                else if (mouse_button.pos.y - rect.min_y()).abs() <= 10. && (mouse_button.pos.x - rect.max_x()).abs()<= 10. {
                    data.cursor.typ = Cursor::NotAllowed;
                    data.cursor.over = Some(Direction::UpRight);
                }
                else if (mouse_button.pos.x - rect.max_x()).abs() <= 10. {
                    if mouse_button.pos.y > rect.min_y() && mouse_button.pos.y < rect.max_y(){
                        // cambia cursore -> sinistra
                        data.cursor.typ = Cursor::ResizeLeftRight;
                        data.cursor.over = Some(Direction::Left);
                    }
                }
                else if (mouse_button.pos.x - rect.min_x()).abs() <= 10. {
                    if mouse_button.pos.y > rect.min_y() && mouse_button.pos.y < rect.max_y(){
                        // cambia cursore -> destra
                        data.cursor.typ = Cursor::ResizeLeftRight;
                        data.cursor.over = Some(Direction::Right);
                    }
                }
                else if (mouse_button.pos.y - rect.max_y()).abs() <= 10. {
                    if mouse_button.pos.x > rect.min_x() && mouse_button.pos.x < rect.max_x(){
                        // cambia cursore -> verticale
                        data.cursor.typ = Cursor::ResizeUpDown;
                        data.cursor.over = Some(Direction::Down);
                    }
                }
                else if (mouse_button.pos.y - rect.min_y()).abs() <= 10. {
                    if mouse_button.pos.x > rect.min_x() && mouse_button.pos.x < rect.max_x(){
                        // cambia cursore -> verticale
                        data.cursor.typ = Cursor::ResizeUpDown;
                        data.cursor.over = Some(Direction::Up);
                    }
                }
                else{
                    data.cursor.typ = Cursor::Arrow;
                    data.cursor.over = None;
                }
                ctx.set_cursor(&data.cursor.typ);
            }
            //sposto premendo
            else if data.cursor.down == true {
                println!("{}-{}-{}-{}", data.rect.start_point.unwrap(), data.rect.p2.unwrap(), data.rect.p3.unwrap(), data.rect.end_point.unwrap());
                match data.cursor.over {
                    Some(Direction::Up) => {
                        data.rect.start_point.replace(Point::new(data.rect.start_point.unwrap().x, mouse_button.pos.y ));
                        data.rect.p2.replace(Point::new(data.rect.p2.unwrap().x, mouse_button.pos.y ));
                    },
                    Some(Direction::Down) => {
                        data.rect.end_point.replace(Point::new(data.rect.end_point.unwrap().x, mouse_button.pos.y ));
                        data.rect.p3.replace(Point::new(data.rect.p3.unwrap().x, mouse_button.pos.y ));
                    },
                    Some(Direction::Left) => {
                        data.rect.start_point.replace(Point::new(mouse_button.pos.x, data.rect.start_point.unwrap().y));
                        data.rect.p3.replace(Point::new(mouse_button.pos.x, data.rect.p3.unwrap().y));
                    },
                    Some(Direction::Right) => {
                        data.rect.end_point.replace(Point::new(mouse_button.pos.x, data.rect.end_point.unwrap().y));
                        data.rect.p2.replace(Point::new(mouse_button.pos.x, data.rect.p2.unwrap().y));
                    },
                    Some(Direction::UpLeft) => {
                        data.rect.start_point.replace(Point::new(mouse_button.pos.x, mouse_button.pos.y));
                        data.rect.p2.replace(Point::new(data.rect.p2.unwrap().x, mouse_button.pos.y));
                        data.rect.p3.replace(Point::new(mouse_button.pos.x, data.rect.p3.unwrap().y));
                    },
                    Some(Direction::UpRight) => {
                        data.rect.start_point.replace(Point::new(data.rect.start_point.unwrap().x, mouse_button.pos.y));
                        data.rect.p2.replace(Point::new(mouse_button.pos.x, mouse_button.pos.y));
                        data.rect.end_point.replace(Point::new(mouse_button.pos.x, data.rect.end_point.unwrap().y));
                    },
                    Some(Direction::DownLeft) => {
                        data.rect.start_point.replace(Point::new(mouse_button.pos.x, data.rect.start_point.unwrap().y));
                        data.rect.p3.replace(Point::new(mouse_button.pos.x, mouse_button.pos.y));
                        data.rect.end_point.replace(Point::new(data.rect.end_point.unwrap().x, mouse_button.pos.y));
                    },
                    Some(Direction::DownRight) => {
                        data.rect.p2.replace(Point::new(mouse_button.pos.x, data.rect.p2.unwrap().y));
                        data.rect.p3.replace(Point::new(data.rect.p3.unwrap().x, mouse_button.pos.y));
                        data.rect.end_point.replace(Point::new(mouse_button.pos.x, mouse_button.pos.y));
                    },
                    None => return, // non è sopra il bordo
                }
            }
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
                .window_size((300.0, 200.0));
            ctx.new_window(new_win);
        }
        Handled::No
    }
}
