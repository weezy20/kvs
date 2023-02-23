//! This builds the `kvs` executable

#[derive(clap::Parser)]
#[command(author, version, about)]
pub struct KvsCLI {
    #[command(subcommand)]
    action: Option<Subcommand>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Set new value at key
    Set {
        /// set key
        #[arg(long)]
        key: String,
        /// value
        #[arg(long)]
        value: String,
    },
    /// Get value at key
    Get {
        /// set key
        #[arg(long)]
        key: String,
    },
    /// Remove value at key
    Rm {
        /// set key
        #[arg(long)]
        key: String,
    },
}

fn main() {
    let cli = <KvsCLI as clap::Parser>::parse();
    println!("Hello, world!");
}
