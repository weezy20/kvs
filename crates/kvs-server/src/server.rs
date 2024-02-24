use kvs::exit_program;
use request::serve_request;
use std::net::{SocketAddr, TcpListener};
use tracing::{error, info};

mod request;
#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    ::env_logger::init();
    let KvsServer { socket, engine } = <KvsServer as clap::Parser>::parse();
    let socket: SocketAddr = socket.parse().expect("Failed to parse socket address");
    let engine: Backend = match engine.expect("clap default used").as_str() {
        "kvs" => Backend::Kvs,
        "sled" => Backend::Sled,
        _ => {
            error!("Unsupported Engine");
            exit_program(2);
        }
    };
    info!("Starting KVS server version {}", env!("CARGO_PKG_VERSION"));
    info!("Server configuration - IP:PORT: {socket}, Storage Engine: {engine}");

    let server = TcpListener::bind(socket).expect("Failed to bind to socket");
    for stream in server.incoming() {
        let request_id = uuid::Uuid::new_v4();
        let span = tracing::info_span!("Request Processing", %request_id);
        let _span_enter = span.enter();
        if let Err(err) = serve_request(stream?) {
            error!(%err)
        }
    }
    Ok(())
}
#[derive(clap::Parser)]
struct KvsServer {
    #[arg(long = "addr", short = 'a', default_value = "127.0.0.1:4000")]
    // Socket v4 or v6 -> IP:PORT
    socket: String,
    #[arg(long, short, default_value = "kvs")]
    /// KV backend to use.
    engine: Option<String>,
}
enum Backend {
    Kvs,
    Sled,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Backend::Kvs => write!(f, "Kvs"),
            Backend::Sled => write!(f, "Sled"),
        }
    }
}
