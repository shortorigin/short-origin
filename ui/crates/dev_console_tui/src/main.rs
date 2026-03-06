fn main() {
    if let Err(error) = dev_console_tui::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
