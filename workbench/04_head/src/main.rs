fn main() {
    if let Err(e) = headr::get_args().and_then(headr::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
