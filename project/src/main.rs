use druid::PlatformError;
use druid::{AppLauncher, LocalizedString, WindowDesc};

mod ui;
use ui::build_ui;

mod data;
use data::AppState;

fn main() -> Result<(), PlatformError> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");

    let width = display_info[0].width as f32 * display_info[0].scale_factor;
    let height = display_info[0].height as f32 * display_info[0].scale_factor;

    let app_state = AppState::new(width, height, display_info[0].scale_factor);

    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Screen grabbing"))
        .window_size((500.0, 500.0));

    AppLauncher::with_window(main_window).launch(app_state)
}
