fn main() {
    env_logger::init();

    if let Err(e) = fortuner::get_args().and_then(fortuner::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
