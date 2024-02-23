#!/bin/bash

cargo build --workspace;

cargo r -p kvs-server -- --engine kvs