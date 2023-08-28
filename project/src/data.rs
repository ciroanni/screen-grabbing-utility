use druid::{widget::Controller, Data, Env, Event, EventCtx, Lens, Widget};
use druid_shell::keyboard_types::{KeyboardEvent, Modifiers, ShortcutMatcher};

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

#[derive(Clone, Data, PartialEq, Lens)]
pub struct AppState {
    pub name: String,
    pub selected_format: ImageFormat,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            name: "".to_string(),
            selected_format: ImageFormat::Jpeg,
        }
    }
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

impl AppState {
    pub fn screen(&mut self) {
        let a = screenshots::DisplayInfo::all();

        let display_info = match a {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        //println!("{:?}", display_info);

        let b = screenshots::Screen::new(&display_info[0]);

        //println!("{:?}", b);

        let c = b.capture();
        //let d=b.capture_area(0, 0, 100, 100);

        let image = match c {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        let scale_factor = display_info[0].scale_factor;
        let width = display_info[0].width as f32;
        let height = display_info[0].height as f32;

        if self.name.is_empty() {
            self.name = "screenshot".to_string();
        }

        let e = image::save_buffer_with_format(
            self.name.as_str().to_owned() + &self.selected_format.to_string(),
            image.rgba(),
            (width * scale_factor) as u32,
            (height * scale_factor) as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png, //useless, but necessary to support formats like gif and webp (save_buffer not working)
        );

        self.name = "".to_string();

        match e {
            Err(why) => return println!("errore:{}", why),
            Ok(()) => return,
        };
    }
}

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
                Modifiers::ALT,
                's',
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
