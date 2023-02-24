//! This builds the `kvs` executable
mod cli;

fn main() {
    use cli::*;
    let cli = <KvsCLI as clap::Parser>::parse();
    if let Some(action) = cli.action {
        match action {
            Action::Set(SetCmd { key, value }) => println!("setting {key} to {value}"),
            Action::Get(GetCmd { key }) => println!("Fetching @ {key}"),
            Action::Rm(RmCmd { key }) => println!("Removing {key}"),
        }
    }
    println!("Hello, world!");
}
