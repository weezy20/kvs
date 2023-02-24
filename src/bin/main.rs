//! This builds the `kvs` executable
use kvs::cli;
fn main() {
    use cli::*;
    let cli = <KvsCLI as clap::Parser>::parse();
    let mut kvs = kvs::KvStore::new();
    if let Some(action) = cli.action {
        match action {
            Action::Set(SetCmd { key, value }) => {
                println!("setting {key} to {value}");
                kvs.set(key, value);
            }
            Action::Get(GetCmd { key }) => {
                println!("Fetching @ {key}");
                kvs.get(key);
            }
            Action::Rm(RmCmd { key }) => {
                println!("Removing {key}");
                kvs.remove(key).unwrap();
            }
        }
    }
}
