#[derive(clap::Parser)]
#[command(author, version, about)]
pub struct KvsCLI {
    #[command(subcommand)]
    pub action: Option<Action>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(clap::Args)]
/// Set new value at key
pub struct SetCmd {
    #[arg(name = "KEY", help = "Key to be set")]
    pub key: String,
    #[arg(name = "VALUE", help = "Value to be set")]
    pub value: String,
}

#[derive(clap::Parser)]
/// Get value at key
pub struct GetCmd {
    #[arg(name = "KEY", help = "Key to be fetch")]
    pub key: String,
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
