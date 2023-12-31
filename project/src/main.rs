use druid::PlatformError;
use druid::{AppLauncher, ImageBuf, LocalizedString, WindowDesc};

mod ui;
use ui::{build_ui, make_menu};

mod data;
use data::{AppState, Delegate};


fn main() -> Result<(), PlatformError> {

    let display_info = screenshots::DisplayInfo::all().expect("Error finding display");

    let app_state = AppState::new(display_info[0].scale_factor, ImageBuf::empty());

    let main_window = WindowDesc::new(build_ui(display_info[0].scale_factor))
    .menu(make_menu)
    .title(LocalizedString::new("Cattura schermo"))
    .window_size((1000.0, 500.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate) //per far funzionare il delegate e quindi i menù a tendina
        .launch(app_state)


}