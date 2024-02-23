use kvs::cli::{Action, GetCmd, RmCmd, SetCmd};
use kvs::exit_program;

fn main() -> anyhow::Result<()> {
    let cli = <Cli as clap::Parser>::parse();
    match cli.action {
        Action::Set(SetCmd { key, value }) => {
            
        },
        Action::Get(GetCmd { key }) => {

        },
        Action::Remove(RmCmd { key }) => {

        },
    }
    exit_program(1);
}

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}
