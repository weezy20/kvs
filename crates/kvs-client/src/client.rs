use kvs::exit_program;

fn main() {
    println!("Hello, world! from kvs client");
    exit_program(1);
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
