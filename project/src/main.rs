use druid::PlatformError;
use druid::{AppLauncher, ImageBuf, LocalizedString, WindowDesc};

mod ui;
use ui::{build_ui, make_menu};

mod data;
use data::{AppState, Delegate};
use std::sync::mpsc::{channel, Receiver, Sender};


fn main() -> Result<(), PlatformError> {

    let display_info = screenshots::DisplayInfo::all().expect("Err");
    let (_sender ,_receiver):(Sender<Vec<u32>>,Receiver<Vec<u32>>)=channel();

    let app_state = AppState::new(display_info[0].scale_factor, ImageBuf::empty());

    let main_window = WindowDesc::new(build_ui(display_info[0].scale_factor))
    .menu(make_menu)
    .title(LocalizedString::new("Screen grabbing"))
    .window_size((1000.0, 500.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate) //per far funzionare il delegate
        .launch(app_state)


}