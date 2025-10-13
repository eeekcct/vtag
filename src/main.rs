mod cli;
mod git;
mod tag;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("❌ {}", e);
        std::process::exit(1);
    }
}
