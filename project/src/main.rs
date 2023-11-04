use data::app_state_derived_lenses::shortcut;
use druid::PlatformError;
use druid::{AppLauncher, ImageBuf, LocalizedString, WindowDesc};

mod ui;
use ui::{build_ui, make_menu};

mod data;
use data::{AppState, Delegate};
use std::sync::mpsc::{self, channel, Receiver, Sender};
use hotkey::*;

fn main() -> Result<(), PlatformError> {
    let display_info = screenshots::DisplayInfo::all().expect("Err");
    let (sender ,receiver):(Sender<Vec<u32>>,Receiver<Vec<u32>>)=channel();

    let app_state = AppState::new(display_info[0].scale_factor, ImageBuf::empty());

    let main_window = WindowDesc::new(build_ui(display_info[0].scale_factor, app_state.img.clone()))
    .menu(make_menu)
    .title(LocalizedString::new("Screen grabbing"))
    .window_size((1000.0, 500.0));

    /*
    std::thread::spawn(move ||{
        create_listener(receiver);
    });
    */

    AppLauncher::with_window(main_window)
        .delegate(Delegate) //per far funzionare il delegate
        .launch(app_state)
}



fn wrap_listener(receiver:Receiver<u32>,keys :Vec<u32>){

    std::thread::spawn(move||{
        for i in 0..2{
        let mut hk = hotkey::Listener::new();
        println!("{:?}",keys);
        hk.register_hotkey(
            keys[0],
            keys[1],
            || {
                println!("Ctrl-Shift-A pressed!")
            },
        )
        .unwrap();

        println!("listener");

        hk.listen();
    }
    }); 

    let hotkeys=receiver.recv();
    println!("wrapper");


}