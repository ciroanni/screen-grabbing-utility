use druid::widget::{Button, CrossAxisAlignment, Flex, Label, TextBox};
use druid::PlatformError;
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

mod ui;
use ui::build_ui;

mod data;
use data::AppState;

#[derive(Clone, Data, Lens)]
struct AppState {
    name: String,
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Screen grabbing"))
        .window_size((300.0, 200.0));

    AppLauncher::with_window(main_window).launch(AppState { name: "".to_string(), format: ".jpeg".to_string()})
}
