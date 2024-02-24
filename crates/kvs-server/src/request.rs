use std::{
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::Context;
use common::Response;
use prost::Message;

pub(crate) fn serve_request(mut s: TcpStream) -> anyhow::Result<()> {
    let mut buffer = vec![0_u8; 1024];
    let bytes_read = s.read(&mut buffer)?;
    // let _req =
    //     common::Get::decode(&buffer[..bytes_read]).context("Unimplemented decode. Use only GET")?;

    // s.shutdown(std::net::Shutdown::Read)?;
    let response = Response {
        value: "OOOONF YOU SENT A GET ".to_string(),
    };
    buffer.clear();
    response.encode(&mut buffer)?;
    s.write_all(buffer.as_slice())?;
    s.flush()?;
    s.shutdown(std::net::Shutdown::Write)?;
    tracing::info!("served :)");
    Ok(())
}
