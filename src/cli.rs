//! CLI machinery for KvStore client

use serde::{Deserialize, Serialize};

#[derive(clap::Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help = true)]
/// The main CLI entry point
pub struct KvsCLI {
    #[command(subcommand)]
    /// Action for the KvStore
    pub action: Option<Action>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(clap::Args, Serialize, Deserialize)]

/// Set new value at key
pub struct SetCmd {
    #[arg(name = "KEY", help = "Key to be set")]
    /// Key to Set
    pub key: String,
    /// Value to Set
    #[arg(name = "VALUE", help = "Value to be set")]
    pub value: String,
}

#[derive(Serialize, Deserialize, clap::Parser)]
/// Get value at key
pub struct GetCmd {
    #[arg(name = "KEY", help = "Key to be fetch")]
    /// Key to fetch
    pub key: String,
}

#[derive(clap::Parser, Serialize, Deserialize)]
/// Remove value at key
pub struct RmCmd {
    #[arg(name = "KEY", help = "Key to be remove")]
    /// Remove value at key
    pub key: String,
}

#[derive(Serialize, Deserialize, clap::Subcommand)]
#[command(subcommand_required = true)]
#[serde(rename = "")]
/// Action Subcommand
pub enum Action {
    /// Set new value at key
    #[serde(rename = "SET")]
    Set(SetCmd),
    /// Get value at key
    #[serde(skip_serializing)]
    Get(GetCmd),
    /// Remove value at key
    #[serde(rename = "RM")]
    Remove(RmCmd),
}
