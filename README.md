# ipfs-resolver

Umbrella project for all things related to resolving and analyzing IPFS blocks with Rust.

Most of the sub-projects have their own README to explain some things in more detail.

## Sub-projects

### `cid-converter`

Inactive. This was a binary project used to convert CIDs in the database from strings to byte arrays.

### `cid-decode`

A binary that reads many CIDs and prints various counts about them.

### `common`

This library package holds basic building blocks used in all other packages, most of all logging and very basic types.
This also contains the code for simulating the BitSwap engine.

### `csv-to-graph`

A binary that converts CSV exports from the database (blocks and references) to a graph in KONECT format.
Due to the size of the exports, this is done incrementally and on-disk.

### `db`

This library package deals with all things db-related.
Specifically, it holds the schemas, types, and functions used by all other packages to interact with the database.

### `db-exporter`

This binary package implements a small tool that tracks the number of records in the database and exports them via
Prometheus.

### `gateway-finder`

This is a binary that identifies public gateways on the overlay network.
It downloads the list of public gateways off github, crafts CIDs, queries them, and listens for BitSwap messages for these CIDs.

### `ipfs-json-to-csv`

This is a binary tool to convert logged BitSwap messages and connection events to CSV data to be analyzed in R.
It tracks connection durations and simulates the BitSwap engine.

### `ipfs-walk-tree`

This binary tool (with a nice terminal UI) uses the database to explore the IPFS DAG.
It can traverse the DAG downwards as well as upwards, if we have parent blocks indexed.

### `resolver`

This binary package produces the actual resolver.
It interacts with an IPFS node via HTTP, parses IPFS blocks' protobuf, and finally puts the results in a database.

### `wantlist-client` and `wantlist-client-lib`

This binary package implements a TCP client to the TCP server implemented in `go-bitswap`.
This makes it possible to receive and process wantlist messages in real-time.
Additionally, the binary runs a prometheus server to publish metrics about the number of messages received.

## Building

You'll need the latest stable Rust.
You'll also need `protoc`, the protocol buffer compiler, from Google, somewhere on your `PATH`.
Then just:

```
cargo build --release
```

This will take a while for the first build.
Also, this will build all sub-projects, which then end up in the `target/` directory of the root project.

## Configuration

All of the packages are configured via environment variables/`.env` files.
The packages contain README files that detail their configuration, and an example, complete `.env` file is given in
[.env](.env).
