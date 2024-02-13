use kvs::{
    cli::{Action, GetCmd, RmCmd, SetCmd},
    exit_program,
};
fn main() {
    let KvsServer {
        action,
        socket,
        engine,
    } = <KvsServer as clap::Parser>::parse();

    println!("Starting KVS server on {socket}");
}
#[derive(clap::Parser)]
struct KvsServer {
    #[clap(subcommand)]
    action: Action,
    #[arg(long = "addr", short = 'a', default_value = "127.0.0.1:4000")]
    // Socket v4 or v6 -> IP:PORT
    socket: String,
    #[arg(long, short, default_value = "kvs")]
    /// KV backend to use.
    engine: Option<String>,
}
