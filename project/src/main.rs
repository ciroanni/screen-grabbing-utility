use druid::PlatformError;
use druid::{AppLauncher, LocalizedString, WindowDesc};

mod ui;
use ui::build_ui;

mod data;
use data::AppState;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Screen grabbing"))
        .window_size((300.0, 200.0));

    AppLauncher::with_window(main_window).launch(AppState::default())
}
