fn main() {
    env_logger::init();

    if let Err(e) = wcr::get_args().and_then(wcr::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
