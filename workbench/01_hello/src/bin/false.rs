fn main() {
    std::process::exit(1);
    // std::process::abort(); // which is not the same as exit(1), it's a panic. will not clean up resources.
}
