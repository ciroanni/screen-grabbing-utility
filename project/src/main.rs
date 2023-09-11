use druid::PlatformError;
use druid::{AppLauncher, LocalizedString, WindowDesc, ImageBuf};

mod ui;
use ui::build_ui;

mod data;
use data::AppState;

fn main() -> Result<(), PlatformError> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let app_state = AppState::new(display_info[0].scale_factor , ImageBuf::empty());

    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Screen grabbing"))
        .window_size((500.0, 500.0));

    AppLauncher::with_window(main_window).launch(app_state)
}
