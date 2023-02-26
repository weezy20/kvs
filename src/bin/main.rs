//! This builds the `kvs` executable
use kvs::cli;
fn main() {
    use cli::*;
    let cli = <KvsCLI as clap::Parser>::parse();
    let mut kvs = kvs::KvStore::new(/* Should mention some file path on disk */);
    if let Some(action) = cli.action {
        match action {
            Action::Set(SetCmd { key, value }) => {
                println!("setting {key} to {value}");
                let Ok(_) = kvs.set(key, value) else {
                    // Note we are not handling the error variants here
                    non_zero_exit();
                };
            }
            Action::Get(GetCmd { key }) => {
                println!("Fetching @ {key}");
                let Ok(val) = kvs.get(key) else {
                    non_zero_exit();
                };
                println!("{val:?}");
            }
            Action::Rm(RmCmd { key }) => {
                println!("Removing {key}");
                let Ok(_) = kvs.remove(key) else {
                    non_zero_exit();
                };
            }
        }
    } else {
        std::process::exit(1);
    }
}

fn non_zero_exit() -> ! {
    std::process::exit(1)
}
