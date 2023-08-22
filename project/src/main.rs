use druid::widget::{Button, CrossAxisAlignment, Flex, Label, TextBox};
use druid::PlatformError;
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

#[derive(Clone, Data, Lens)]
struct AppState {
    name: String,
}

fn build_ui() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            TextBox::new()
                .with_placeholder("screen.jpeg")
                .expand_width()
                .lens(AppState::name)
        )
        .with_spacer(20.0)
        .with_child(Button::new("+ Nuovo").on_click(|ctx, data: &mut AppState, _env| {
            data.screen();
        }))
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Rust Druid Example"))
        .window_size((300.0, 200.0));

    AppLauncher::with_window(main_window).launch(AppState { name: "".to_string()})
}

impl AppState {
    fn screen(&mut self) {
        let a = screenshots::DisplayInfo::all();

        let display_info = match a {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        println!("{:?}", display_info);

        let b = screenshots::Screen::new(&display_info[0]);

        println!("{:?}", b);

        let c = b.capture();
        //let d=b.capture_area(0, 0, 100, 100);

        let image = match c {
            Err(why) => return println!("{}", why),
            Ok(info) => info,
        };

        let ok = image.to_png(None);

        let immagine = match ok {
            Err(why) => return println!("{}", why),
            Ok(data) => data,
        };

        println!("lunghezza vettore immagine:{}", immagine.len());

        let f = image::guess_format(&immagine);
        let format = match f {
            Err(why) => return println!("{}", why),
            Ok(data) => data,
        };

        println!("{:?}", format);

        let scale_factor = display_info[0].scale_factor;
        let width = display_info[0].width as f32;
        let height = display_info[0].height as f32;


        let e = image::save_buffer_with_format(
            self.name.as_str(),
            image.rgba(),
            (width * scale_factor) as u32,
            (height * scale_factor) as u32,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        );
        match e {
            Err(why) => return println!("errore:{}", why),
            Ok(()) => return,
        };
    }
}