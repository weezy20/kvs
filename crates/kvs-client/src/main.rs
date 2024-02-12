fn main() {
    println!("Hello, world! from kvs client");
    exit_program(0);
}
/// Non-zero exit code indicates a program error
fn exit_program(code: i32) -> ! {
    std::process::exit(code)
}

#[derive(Debug, clap::Parser)]
struct Cli {
    #[arg(short, long)]
    get: String,
    #[arg(short, long)]
    set: String,
    #[arg(short, long)]
    rm: String,
}
