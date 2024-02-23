use std::net::TcpStream;

pub(crate) fn serve_request(s: TcpStream) {
    tracing::info!("served :)")
}
