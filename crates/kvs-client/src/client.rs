use anyhow::Context;
use common::message::Payload;
use common::{Get, Message, Rm, Set, MessageType};
use kvs::cli::{Action, GetCmd, RmCmd, SetCmd};
use kvs::exit_program;
use log::trace;
use prost::Message as ProstMessage;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

fn main() -> anyhow::Result<()> {
    ::env_logger::init();
    let cli = <Cli as clap::Parser>::parse();
    let server = cli.addr.parse::<SocketAddr>()?;
    let mut server = TcpStream::connect(server)?;
    log::info!("🌐 Connected to server [{}]", server.peer_addr()?);
    if match cli.action {
        Action::Set(SetCmd { key, value }) => {
            log::debug!("✉️ Requesting -> Set {} = {}", key, value);
            send(Payload::Set(Set { key, value }), &mut server)
        }
        Action::Get(GetCmd { key }) => {
            log::debug!("✉️ Requesting -> Get {}", key);
            send(Payload::Get(Get { key }), &mut server)
        }
        Action::Remove(RmCmd { key }) => {
            log::debug!("✉️ Requesting -> Rm {}", key);
            send(Payload::Rm(Rm { key }), &mut server)
        }
    }
    .is_err()
    {
        exit_program(1);
    }
    exit_program(0);
}

fn send(payload: Payload, server: &mut TcpStream) -> anyhow::Result<()> {
    let mut message_bytes : Vec<u8> = vec![];
    let r#type = match payload {
        Payload::Set { .. } => MessageType::Set as i32, // 0
        Payload::Get { .. } => MessageType::Get as i32, // 1
        Payload::Rm { .. } => MessageType::Rm as i32, // 2
    };
    let message = Message {
        r#type,
        payload: Some(payload),
    };
    trace!("Message request -> {:#?}", message);
    {
        // Send message towards server
        message
            .encode(&mut message_bytes)
            .context("failed to encode message into bytes")?;
        server.write_all(&message_bytes)?;
        server.flush()?;
        log::debug!("Written {} bytes to server stream", message_bytes.len());
        log::trace!("Bytes -> {message_bytes:?}");
        server.shutdown(std::net::Shutdown::Write)?;
    }
    // Clear buffer, await response
    message_bytes.clear();
    {
        // We depend on the server to shutdown the stream after it's finished sending a response
        let bytes_read = server.read_to_end(&mut message_bytes)?;
        log::debug!("Got {} bytes back ", bytes_read);
        let response = common::Response::decode(&message_bytes[0..bytes_read])
            .context("failed to decode message response from server")?;
        println!("{}", response.value);
    }
    Ok(())
}

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
    /// Server location
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    addr: String,
}
