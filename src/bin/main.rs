//! This builds the `kvs` executable

#[derive(clap::Parser)]
#[command(author, version, about)]
pub struct KvsCLI {
    #[command(subcommand)]
    action: Option<Action>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(clap::Args)]
/// Set new value at key
pub struct SetCmd {
    #[arg(name = "KEY", help = "Key to be set")]
    key: String,
    #[arg(name = "VALUE", help = "Value to be set")]
    value: String,
}

#[derive(clap::Parser)]
/// Get value at key
pub struct GetCmd {
    #[arg(name = "KEY", help = "Key to be fetch")]
    key: String,
}

#[derive(clap::Parser)]
/// Remove value at key
pub struct RmCmd {
    #[arg(name = "KEY", help = "Key to be remove")]
    pub key: String,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Set new value at key
    Set(SetCmd),
    /// Get value at key
    Get(GetCmd),
    /// Remove value at key
    Rm(RmCmd),
}

fn main() {
    let cli = <KvsCLI as clap::Parser>::parse();
    if let Some(action) = cli.action {
        match action {
            Action::Set(SetCmd { key, value }) => println!("setting {key} to {value}"),
            Action::Get(GetCmd { key }) => println!("Fetching @ {key}"),
            Action::Rm( RmCmd { key }) => println!("Removing {key}"),
        }
    }
    println!("Hello, world!");
}
