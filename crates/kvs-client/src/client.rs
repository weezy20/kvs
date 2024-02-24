use anyhow::Context;
use common::{Get, Rm, Set};
use kvs::cli::{Action, GetCmd, RmCmd, SetCmd};
use kvs::exit_program;
use prost::Message;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

fn main() -> anyhow::Result<()> {
    ::env_logger::init();
    let cli = <Cli as clap::Parser>::parse();
    let server = cli.server.parse::<SocketAddr>()?;
    let mut server = TcpStream::connect(server)?;
    log::info!("connected to server {}", server.peer_addr()?);
    match cli.action {
        Action::Set(SetCmd { key, value }) => {
            log::debug!("Requesting -> Set {} = {}", key, value);
            send(Set { key, value }, &mut server)?;
        }
        Action::Get(GetCmd { key }) => {
            log::debug!("Requesting -> Get {}", key);
            send(Get { key }, &mut server)?;
        }
        Action::Remove(RmCmd { key }) => {
            log::debug!("Requesting -> Rm {}", key);
            send(Rm { key }, &mut server)?;
        }
    }
    exit_program(0);
}

fn send(message: impl prost::Message, server: &mut TcpStream) -> anyhow::Result<()> {
    // Send message towards server
    let mut message_bytes = vec![0_u8; 1024];
    message
        .encode(&mut message_bytes)
        .context("failed to encode message into bytes")?;
    server.write_all(&message_bytes)?;
    server.flush()?;
    log::debug!("Written {} bytes to server stream", message_bytes.len());
    // Clear buffer, await response
    message_bytes.clear();
    let bytes_read = server.read_to_end(&mut message_bytes)?;
    log::debug!("Got {} bytes back ", bytes_read);
    let response = common::Response::decode(&message_bytes[0..bytes_read])
        .context("failed to decode message response from server")?;
    println!("{}", response.value);
    Ok(())
}

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
    /// Server location
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    server: String,
}
