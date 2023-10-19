//! This builds the `kvs` executable
use kvs::cli;
use log::{error, info};
use std::env;
fn main() -> kvs::Result<()> {
    ::dotenv::dotenv().ok();
    // Read kv_00001.log file into BufReader 
    // let file = std::fs::File::open("kv_00001.log")?;
    // let buf = std::io::BufReader::new(&file);
    // let buf2 = std::io::BufRead::lines(buf)
    //             .map(|line| line.unwrap())
    //             .nth(1)
    //             .expect("contents cannot be empty");
    // // print buf2
    // println!("{}", buf2);
    // std::process::exit(0);
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
                info!("Setting {key} to {value}");
                let Ok(_) = kvs.set(key, value) else {
                    // Note we are not handling the error variants here
                    exit_program(0);
                };
            }
            Action::Get(GetCmd { key }) => {
                info!("Fetching {key}");
                let val = kvs.get(key)?;
                if let Some(v) = val {
                    println!("{v}");
                } else {
                    error!("Key not found");
                    // exit_program(1);
                };
            }
            Action::Remove(RmCmd { key }) => {
                info!("Removing \"{key}\"");
                let _  = kvs.remove(key);
                exit_program(0);
            }
        }
        if cli.compact {
            kvs.compaction()?;
        }
        Ok(())
    } else {
        unreachable!("Action (subcommands) are required");
    }
}
/// Non-zero exit code indicates a program error
fn exit_program(code: i32) -> ! {
    std::process::exit(code)
}
