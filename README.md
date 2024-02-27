# KVS - A simple command line key-value datastore

## Purpose

Provide a simple key value store with persistent storage
1. `set <key> <value>`
2. `get <key>`
3. `rm <key>`

## Structure

1. `lib` Implments the main KvStore functionality, swappable backends with `KvStore` and `sled` another high performance key value embeddable database. One can add their own version by implementing `KvsEngine`.

2. `crates/common` Exports protobuf definitions for the client-server communications protocol

3. `crates/kvs-client` and `crates/kvs-server` may run on the same or different machines as long as they can find each other on the network. `kvs-client` essentially performs the same actions a user would perform on the CLI directly when running `lib/src/bin/main.rs` [except running `compaction`, which shouldn't be a client's concern when interacting with the DB].

## Development

The protobuf definitions are found in `crates/common/src/message.proto`. It may be extended or modified as seen fit.

To run, first build the workspace

`./build.sh`

Then run the `kvs-server` with `./server.sh` which runs on `localhost:4000`. You may include custom logging targets.

You may also start it at a different port, or with sled: 

`RUST_LOG=trace cargo r -p kvs-server --addr <Socket> --engine <kvs|sled>`

Finally, interact with it using `./client set foo bar` which if `--addr` is not provided, connects to `localhost:4000` by default

## Tests

`cargo test --workspace`