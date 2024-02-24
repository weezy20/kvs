//! Protobuf definitions for over the wire communication
//! Types generated via the build script and prost/protoc.
//! Running build on this crate will place these generated types under target/<release/debug>/common-<hash>/out/

pub mod msg {
    include!(concat!(env!("OUT_DIR"), "/kvs_message.rs"));
}
pub use msg::*;
