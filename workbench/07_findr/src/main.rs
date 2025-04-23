fn main() {
    env_logger::init();

    if let Err(e) = findr::get_args().and_then(findr::run_v2) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
