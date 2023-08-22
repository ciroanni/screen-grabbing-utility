use druid::widget::{Button, CrossAxisAlignment, Flex, Label};
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
                .lens(AppState::new_todo),
        )
        .with_spacer(20.0)
        .with_child(Button::new("+ Nuovo").on_click(|ctx, data: &mut AppState, _env| {}))
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui())
        .title(LocalizedString::new("Rust Druid Example"))
        .window_size((300.0, 200.0));

    AppLauncher::with_window(main_window).launch(AppState { counter: 0 })
}
