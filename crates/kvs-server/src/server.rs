use kvs::{exit_program, KvStore, SledKvsEngine};
use request::serve_request;
use std::env;
use std::net::{SocketAddr, TcpListener};
use tracing::{error, info};

mod request;
#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    ::env_logger::init();
    let KvsServer { socket, engine } = <KvsServer as clap::Parser>::parse();
    let socket: SocketAddr = socket.parse().expect("Failed to parse socket address");
    let engine_str = engine.expect("clap default used");
    let mut backend: Backend = match engine_str.to_lowercase().as_str() {
        "kvs" => Backend::Kvs(KvStore::open(env::current_dir()?)?),
        "sled" => Backend::Sled(SledKvsEngine::open(env::current_dir()?)?),
        _ => {
            error!("Unsupported Engine");
            exit_program(2);
        }
    };
    info!("Starting KVS server version {}", env!("CARGO_PKG_VERSION"));
    info!(
        "Server configuration - IP:PORT: {socket}, Storage Engine: {}",
        engine_str
    );

    let server = TcpListener::bind(socket).expect("Failed to bind to socket");
    for stream in server.incoming() {
        let request_id = uuid::Uuid::new_v4();
        let span = tracing::info_span!("Request Processing", %request_id);
        let _span_enter = span.enter();
        if let Err(err) = serve_request(&mut backend, stream?) {
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
    Kvs(KvStore),
    Sled(SledKvsEngine),
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Backend::Kvs(_) => write!(f, "Kvs"),
            Backend::Sled(_) => write!(f, "Sled"),
        }
    }
}

impl std::ops::Deref for Backend {
    type Target = dyn kvs::KvsEngine;
    fn deref(&self) -> &Self::Target {
        match self {
            Backend::Kvs(kvs) => kvs,
            Backend::Sled(sled) => sled,
        }
    }
}
impl std::ops::DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Backend::Kvs(kvs) => kvs,
            Backend::Sled(sled) => sled,
        }
    }
}
