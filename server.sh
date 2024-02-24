#!/bin/bash
export RUST_LOG=debug
cargo build --workspace;

cargo watch --watch crates/kvs-server -x 'r -p kvs-server -- --engine kvs'