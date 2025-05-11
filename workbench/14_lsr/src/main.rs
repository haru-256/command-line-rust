fn main() {
    env_logger::init();

    if let Err(e) = lsr::get_args().and_then(lsr::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
