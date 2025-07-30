fn main() {
    let success = satchel_demo::discover_and_run();
    std::process::exit(if success { 0 } else { 1 });
}
