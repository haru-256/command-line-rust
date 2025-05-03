fn main() {
    env_logger::init();

    if let Err(err) = commr::get_args().and_then(commr::run) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
