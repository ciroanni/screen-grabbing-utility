use druid::PlatformError;
use druid::{AppLauncher, ImageBuf, LocalizedString, WindowDesc};

mod ui;
use ui::build_ui;

mod data;
use data::{AppState, Delegate};

fn main() -> Result<(), PlatformError> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let app_state = AppState::new(display_info[0].scale_factor, ImageBuf::empty());

    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Screen grabbing"))
        .window_size((500.0, 500.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate) //per far funzionare il delegate
        .launch(app_state)
}
