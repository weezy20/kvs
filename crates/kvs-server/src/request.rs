use crate::Backend;
use anyhow::{anyhow, bail, Context};
use common::{message::Payload, Get, Message, Response, Rm, Set};
use kvs::DbError;
use prost::Message as ProstMessage;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace};

pub(crate) fn serve_request(backend: &mut Backend, mut stream: TcpStream) -> anyhow::Result<()> {
    // Note: If you're using `read_to_end`, you can simply use a 0-length `vec![]` that will be coerced to the
    // correct length during read. Say N bytes are read, so the vec will be N bytes long
    // In this case however, we write our code using 1024 bytes and use the `bytes_read` a crucial variable
    // to slice into the buffer for correct decoding of the protobuf.
    let mut buffer: Vec<u8> = vec![0_u8; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    trace!("{bytes_read} bytes read : {:?}", &buffer[..bytes_read]);
    if buffer.iter().all(|x| *x == 0) {
        bail!("Request is zeroes 0️.. aborting");
    }
    {
        /* Response */
        let response: Vec<u8> = handle_request(backend, &buffer[..bytes_read])?;
        drop(buffer);
        stream.write_all(response.as_slice())?;
        stream.flush()?;
        stream.shutdown(Shutdown::Write)?;
        trace!("Request completed 🚀");
    }
    Ok(())
}
// This functions returns a Result, whose Err variant is supposed to notify our server
// That some processing has failed. Kvs Backend errors are handled differently in that
// the failure is logged, and the client is notified with a Response { success: false }
fn handle_request(backend: &mut Backend, buffer: &[u8]) -> anyhow::Result<Vec<u8>> {
    trace!("🔄 Processing request");
    // Note, the type of request is embedded both in the `type` and `payload` fields of `Message`
    let request: Message = Message::decode(buffer).with_context(|| {
        error!("🚨 Failed to parse request from client",);
        "🚨 Server cannot decode request"
    })?;
    let payload = request
        .payload
        .ok_or(anyhow!("🚨 Missing payload in Request"))?;
    // No matter error or success, we create a response to send back to the client
    let response = match payload {
        Payload::Set(Set { key, value }) => {
            trace!("🔄 Processing Set {key}->{value} request");
            match backend.set(key, value) {
                Ok(()) => Response {
                    success: true,
                    value: None,
                },
                // A backend Err indicates that our KVS failed but we must also notify
                // this to the client. We follow this logic with all other arms
                Err(e) => {
                    error!("🚨 Backend failed to SET key-value pair: {}", e);
                    Response {
                        success: false,
                        value: None,
                    }
                }
            }
        }
        Payload::Get(Get { key }) => {
            trace!("🔄 Processing Get {key} request");
            match backend.get(key) {
                Ok(Some(value)) => Response {
                    success: true,
                    value: Some(value),
                },
                Ok(None) => Response {
                    success: true,
                    value: Some(format!("Key not found")),
                },
                Err(e) => {
                    error!("🚨 Backend failed to GET key-value pair: {}", e);
                    Response {
                        success: false,
                        value: None,
                    }
                }
            }
        }
        Payload::Rm(Rm { key }) => {
            trace!("🔄 Processing Remove {key} request");
            match backend.remove(key) {
                Ok(()) => Response {
                    success: true,
                    value: None,
                },
                Err(e) => {
                    error!("🚨 Backend failed to RM key-value pair: {}", e);
                    Response {
                        success: false,
                        // TODO: How can we match on e if DbError doesn't implement PartialEq?
                        value: match e {
                            DbError::KeyNotFound => Some(format!("Key not found")),
                            _ => None,
                        },
                    }
                }
            }
        }
    };
    let mut buffer: Vec<u8> = vec![];
    response
        .encode(&mut buffer)
        .context("Server failed to encode response back to client")?;
    Ok(buffer)
}
