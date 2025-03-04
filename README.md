# Malachite-Eth

A shim layer on top of [Malachite][malachite]'s channel-based API that connects the consensus engine
to any Ethereum execution client via [Engine API][engine-api].

## Goals

The goal in this repo is to build an MVP of an integration of Malachite consensus engine with
Ethereum execution clients, in particular Reth.

Rough architecture:
- Use Malachite as a CL (consensus layer client)
- Use Reth as a EL (execution layer client)
- Uses the [channel-based API of Malachite](<[url](https://github.com/informalsystems/malachite/tree/main/code/examples/channel)>) to do the integration between CL and EL
- We want to test the integration at large scale O(100) nodes and quantify the latency and throughput

## Background

To get familiar with the project and its goals, the following resources can be useful:
- The [channel-based application tutorial in Malachite](<[url](https://github.com/informalsystems/malachite/blob/main/docs/tutorials/channels.md)>)
- The proof of concept -- a very naive integration -- Reth x Malachite integration in [rem-poc](<[url](https://github.com/adizere/rem-poc)>)
- Examples from the [reth repo](<[url](https://github.com/paradigmxyz/reth/tree/main/examples)>)

## Development

### Requirements

- Docker
- [Foundry][foundry], to be able to use the [`cast`][cast] tool.

### Setup and run a local testnet

Running `make` will:
- Clean up any previous running testnet, if any.
- Build the app.
- Generate a genesis file in `./assets/genesis.json`.
- Spin up docker containers including 3 Reth servers + monitoring services (Prometheus and Grafana).
- Generate configuration files for 3 Malachite nodes in `./nodes/`.
- Run the Malachite nodes.

If successful, Malachite logs for each node can be found at `nodes/<N>/logs/node.log`.

Check out the metrics in the Grafana dashboards at `http://localhost:3000`.

Using the `cast` tool, one can query Reth to explore the blockchain. For example:
```
cast block-number                      # show the number of the latest finalised block
cast block 3                           # show the block #3's content
cast balances 0x...                    # show the balance of an account
cast rpc txpool_status                 # show number of pending and queued transactions
cast rpc eth_getTransactionCount 0x... # get latest nonce value used for given account
```

### Inject transaction load

In a separate console, run the following command (or `make spam`) to send transactions at a rate of
1000 tx/s to one of the Reth RPC servers.
```
cargo run --bin malachitebft-eth-utils spam --num-txs 1000000 --rate=1000
```

[malachite]: https://github.com/informalsystems/malachite
[engine-api]: https://github.com/ethereum/execution-apis/tree/main/src/engine
[foundry]: https://book.getfoundry.sh/getting-started/installation
[cast]: https://book.getfoundry.sh/cast/
