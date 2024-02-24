use anyhow::{bail, Context};
use common::{Message, MessageType, Response};
use prost::Message as ProstMessage;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace};

pub(crate) fn serve_request(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buffer: Vec<u8> = vec![];
    let bytes_read = stream.read_to_end(&mut buffer)?;
    trace!("{bytes_read} bytes read : {buffer:?}");
    if buffer.iter().all(|x| *x == 0) {
        bail!("Request is zeroes.. aborting");
    }
    stream.shutdown(Shutdown::Read)?;
    {
        /* Response */
        let response: Vec<u8> = handle_request(&buffer)?;
        drop(buffer);
        stream.write_all(response.as_slice())?;
        stream.flush()?;
        stream.shutdown(Shutdown::Write)?;
        trace!("Request completed 🚀");
    }
    Ok(())
}
fn handle_request(buffer: &[u8]) -> anyhow::Result<Vec<u8>> {
    trace!("Handling request");
    let request: Message = Message::decode(buffer).with_context(|| {
        error!("🚨 Failed to parse request from client",);
        "🚨 Server cannot decode request"
    })?;
    let response = match MessageType::try_from(request.r#type)? {
        MessageType::Set => Response {
            value: "OOOONF YOU SENT A SET ".to_string(),
        },
        MessageType::Get => Response {
            value: "GO GET ".to_string(),
        },
        MessageType::Rm => Response {
            value: "O RM ".to_string(),
        },
    };
    let mut buffer: Vec<u8> = vec![];
    response
        .encode(&mut buffer)
        .context("Server failed to encode response back to client")?;
    Ok(buffer)
}
