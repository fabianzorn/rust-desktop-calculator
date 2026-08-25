mod calculator;
mod desktop_ui;

fn main() {
    if let Err(error) = desktop_ui::run() {
        eprintln!("Error while starting the Desktop-UI: {error}");
    }
}
