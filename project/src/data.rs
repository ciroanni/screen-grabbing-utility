use druid::{widget::Controller, Data, Env, Event, EventCtx, Lens, MouseEvent, Point, Widget};
use druid_shell::keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};

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
    fn to_string(&self) -> String {
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

#[derive(Clone, Data, PartialEq, Lens)]
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
}

impl AppState {
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
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
        }
    }


    pub fn screen(&mut self) {
        let a = screenshots::DisplayInfo::all();

        let display_info = match a {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        let b = screenshots::Screen::new(&display_info[0]);

        //let c = b.capture();
        let c = b.capture_area(
            (self.from.x as f32 * self.scale) as i32,
            (self.from.y as f32 * self.scale) as i32,
            (self.size.x as f32 * self.scale) as u32,
            (self.size.y as f32 * self.scale) as u32,
        );

        let image = match c {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        /*let width = display_info[0].width as f32;
        let height = display_info[0].height as f32;*/

        if self.name.is_empty() {
            self.name = "screenshot".to_string();
        }

        let e = image::save_buffer_with_format(
            self.name.as_str().to_owned() + &self.selected_format.to_string(),
            image.rgba(),
            (self.size.x as f32 * self.scale) as u32,
            (self.size.y as f32 * self.scale) as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png, //useless, but necessary to support formats like gif and webp (save_buffer not working)
        );

        *self = AppState::new(self.size.x as u32, self.size.y as u32, self.scale);

        match e {
            Err(why) => return println!("errore:{}", why),
            Ok(()) => return,
        };
    }
}

#[derive(Clone, Data, PartialEq, Lens, Debug)]
pub struct SelectionRectangle {
    pub start_point: Option<Point>,
    pub end_point: Option<Point>,
}

impl Default for SelectionRectangle {
    fn default() -> Self {
        SelectionRectangle {
            start_point: None,
            end_point: None,
        }
    }
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
                || data.screen(),
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

//Controller to take screen after click and drag motion
pub struct AreaController;

impl<W: Widget<AppState>> Controller<AppState, W> for AreaController {
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
            data.from = mouse_down.pos;
        }

        if let Event::MouseUp(mouse_button) = event {
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
            data.size.x = (data.from.x - mouse_up.pos.x).abs();
            data.size.y = (data.from.y - mouse_up.pos.y).abs();
            data.rect = SelectionRectangle::default();
            ctx.window().close();
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

//Controller to paint live the selection rectangle
pub struct PainterController;

impl<W: Widget<AppState>> Controller<AppState, W> for PainterController {
    fn event(
        &mut self,
        _child: &mut W,
        _ctx: &mut EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        _env: &Env,
    ) {
        if let Event::MouseDown(mouse_button) = event {
            data.rect.start_point = Some(mouse_button.pos);
            data.rect.end_point = None;
        } else if let Event::MouseUp(mouse_button) = event {
            data.rect.end_point = Some(mouse_button.pos);
        } else if let Event::MouseMove(mouse_button) = event {
            if !data.rect.start_point.is_none() {
                data.rect.end_point = Some(mouse_button.pos);
            }
        }
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
