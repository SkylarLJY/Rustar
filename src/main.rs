use std::env;
mod rustar;

// implement tar in Rust
fn main() {
    // read commmand line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Not enout arguments.");
        std::process::exit(1);
    }

    rustar::run_tar(&args[1..]).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    });
}
