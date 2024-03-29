use anyhow::bail;
use env_logger::{Builder, Target};
use kvs::{exit_program, KvStore, SledKvsEngine};
use request::serve_request;
use std::env;
use std::ffi::OsString;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use tracing::{error, info};
mod request;
#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    Builder::new()
        .target(Target::Stderr)
        .filter_level(log::LevelFilter::Info)
        .init();
    let KvsServer { socket, engine, .. } = <KvsServer as clap::Parser>::parse();
    let socket: SocketAddr = socket.parse().expect("Failed to parse socket address");
    let engine_str = engine.expect("clap default used");
    let existing_db = match check_db(env::current_dir()?) {
        Ok(db) => db,
        Err(err) => {
            error!("{}", err);
            exit_program(4);
        }
    };
    let mut backend: Backend = match engine_str.to_lowercase().as_str() {
        "kvs" => {
            if existing_db == Db::Sled {
                exit_program(10);
            };
            Backend::Kvs(KvStore::open(env::current_dir()?)?)
        }
        "sled" => {
            if existing_db == Db::Kvs {
                exit_program(11);
            };
            Backend::Sled(SledKvsEngine::open(env::current_dir()?)?)
        }
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
#[command(version)]
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
#[derive(Debug, Default, PartialEq)]
enum Db {
    Sled,
    Kvs,
    #[default]
    None,
}
fn check_db(dir: PathBuf) -> anyhow::Result<Db> {
    if !dir.exists() {
        bail!("Directory does not exist");
    }
    if !dir.is_dir() {
        bail!("Path is not a directory");
    }
    let mut sled_db = false;
    let mut kvs_db = false;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_name() == OsString::from("db") {
                    println!("Sled found");
                    sled_db = true;
                } else if entry.file_name().to_str().unwrap().starts_with("kv_")
                    && entry.file_name().to_str().unwrap().ends_with(".log")
                {
                    println!("Kvs found");
                    kvs_db = true;
                }
            }
        }
    }
    match (sled_db, kvs_db) {
        (true, true) => bail!("Both databases found. Abort"),
        (true, false) => Ok(Db::Sled),
        (false, true) => Ok(Db::Kvs),
        (false, false) => Ok(Db::None),
    }
}
