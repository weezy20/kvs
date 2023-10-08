//! This builds the `kvs` executable
use kvs::cli;
use log::info;
use std::env;
fn main() -> kvs::Result<()> {
    use cli::*;
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();
    let cli = <KvsCLI as clap::Parser>::parse();
    // create a local kvs instance
    let mut kvs = kvs::KvStore::open(env::current_dir()?)?;

    if let Some(action) = cli.action {
        match action {
            Action::Set(SetCmd { key, value }) => {
                info!("setting {key} to {value}");
                let Ok(_) = kvs.set(key, value) else {
                    // Note we are not handling the error variants here
                    non_zero_exit();
                };
            }
            Action::Get(GetCmd { key }) => {
                info!("Fetching @ {key}");
                let Ok(val) = kvs.get(key) else {
                    non_zero_exit();
                };
                println!("{val:?}");
            }
            Action::Rm(RmCmd { key }) => {
                info!("Removing {key}");
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
