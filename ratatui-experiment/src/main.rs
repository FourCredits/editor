mod state;
mod terminal;
mod ui;

use state::App;
use terminal::TerminalUi;
use ui::Ui;

type UiToUse = TerminalUi;
type Error = <UiToUse as Ui>::Error;

fn main() -> Result<(), Error> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        UiToUse::on_panic();
        original_hook(panic);
    }));
    let mut app = App::default();
    let mut ui = UiToUse::setup()?;
    let result = run(&mut app, &mut ui);
    ui.finish()?;
    result
}

fn run(app: &mut App, ui: &mut UiToUse) -> Result<(), Error> {
    while !app.exited {
        ui.render(app)?;
        let input = ui.get_input()?;
        app.accept_input(input);
    }
    Ok(())
}
