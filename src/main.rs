use clap::Parser;
mod git;
mod tag;

#[derive(Parser)]
#[command(author, version, about = "Create and push git tags", long_about = None)]
struct Args {
    /// Tag name
    tag: String,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = tag::run(&args.tag) {
        eprintln!("❌ {}", e);
        std::process::exit(1);
    }
}
