//! This builds the `kvs` executable
use kvs::cli;
use std::env::current_dir;

fn main() -> kvs::Result<()> {
    use cli::*;
    let cli = <KvsCLI as clap::Parser>::parse();
    // create a local kvs instance
    let mut kvs = kvs::KvStore::open(current_dir()?)?;

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
        Ok(())
    } else {
        unreachable!("Action (subcommands) are required");
    }
}

fn non_zero_exit() -> ! {
    std::process::exit(1)
}
