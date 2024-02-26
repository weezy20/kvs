#!/bin/bash
# client.sh
RUST_LOG=info cargo r -p kvs-client "$@"
