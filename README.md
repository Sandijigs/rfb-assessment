# rfb-cli

A command-line application in Rust that talks to a local **Bitcoin Core** node over its JSON-RPC interface. Built for the Btrust *Rust for Bitcoin* take-home assessment — the full brief is in [ASSESSMENT.md](./ASSESSMENT.md).

The CLI targets a regtest node running under [Polar](https://lightningpolar.com/), but the config layer works against any Bitcoin Core node reachable over HTTP.

---

## Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Setting up Polar](#setting-up-polar)
- [Configuration](#configuration)
- [Usage](#usage)
- [Error handling](#error-handling)
- [Project structure](#project-structure)
- [Design decisions](#design-decisions)
- [Bonus features implemented](#bonus-features-implemented)

---

## Prerequisites

| Tool | Version tested |
|---|---|
| Rust (via `rustup`) | 1.94 (edition 2024) |
| Docker Desktop | 27+ |
| Polar | 4.0.0 |

---

## Installation

```bash
git clone <this-repo> rfb-assessment
cd rfb-assessment
cargo build --release
```

For faster iteration during development, use `cargo run -- <subcommand>` instead of building a release binary.

---

## Setting up Polar

Polar spins up Bitcoin/Lightning nodes inside Docker containers.

1. **Install Polar** — download the macOS DMG from [lightningpolar.com](https://lightningpolar.com/) or the [GitHub releases page](https://github.com/jamaljsr/polar/releases), then drag `Polar.app` to `/Applications`. Polar is not signed by Apple; on first launch open **System Settings → Privacy & Security** and click **Open Anyway** if Gatekeeper blocks it.
2. **Start Docker Desktop** and wait for the daemon to be ready.
3. **Create a network** in Polar: click **+ Create Network**, name it (e.g. `rfb-assessment`), and set **Bitcoin Core: 1**, all Lightning nodes: **0**. Submit.
4. **Start the network** — click the big **Start** button. Polar pulls the `polarlightning/bitcoind:30.0` Docker image on first run and boots the container.
5. **Find the RPC credentials** — once the node icon (`backend1`) turns green, click it, then click the **Connect** tab in the right sidebar. Default values on a fresh Polar network:
   - Host: `127.0.0.1`
   - Port: `18443`
   - Username: `polaruser`
   - Password: `polarpass`
6. **(Optional) Mine some blocks** so the wallet has spendable coins. Coinbase rewards need 100 confirmations to mature:

   ```bash
   docker exec polar-n1-backend1 bitcoin-cli \
     -regtest -rpcuser=polaruser -rpcpassword=polarpass -generate 101
   ```

### Sanity check with `curl`

```bash
curl -s --user polaruser:polarpass \
  --data-binary '{"jsonrpc":"1.0","id":"probe","method":"getblockchaininfo","params":[]}' \
  -H 'content-type: text/plain;' \
  http://127.0.0.1:18443/
```

A JSON response with `"chain":"regtest"` confirms the node is reachable and the credentials are valid.

---

## Configuration

The CLI reads its RPC connection settings from three sources, in this order of precedence (highest to lowest):

1. **Command-line flags** — `--rpc-url`, `--rpc-user`, `--rpc-password`, `--wallet`
2. **Environment variables** — `BITCOIN_RPC_URL`, `BITCOIN_RPC_USER`, `BITCOIN_RPC_PASSWORD`, `BITCOIN_WALLET`
3. **TOML config file** — passed with `--config <path>` or `RFB_CONFIG=<path>`
4. **Built-in defaults** — Polar's out-of-the-box values (`http://127.0.0.1:18443`, `polaruser`, `polarpass`)

Because the built-in defaults match Polar's defaults, a fresh Polar network needs **no configuration at all** to run the CLI.

### Example TOML config

```toml
# config.toml
rpc_url      = "http://127.0.0.1:18443"
rpc_user     = "polaruser"
rpc_password = "polarpass"
wallet       = "my-wallet"        # optional
```

```bash
cargo run -- --config ./config.toml blockchain-info
```

### Example environment variables

```bash
export BITCOIN_RPC_URL=http://127.0.0.1:18443
export BITCOIN_RPC_USER=polaruser
export BITCOIN_RPC_PASSWORD=polarpass
cargo run -- blockchain-info
```

---

## Usage

> Command output is colored in a real terminal (chain names in magenta, numeric values in green, addresses in cyan). Colors are automatically stripped when stdout is piped or redirected, and can be disabled globally by setting `NO_COLOR=1`.

```
$ cargo run -- --help
Talk to a Bitcoin Core node over JSON-RPC (Regtest via Polar).

Usage: rfb-cli [OPTIONS] <COMMAND>

Commands:
  blockchain-info  Show chain, blocks, headers, difficulty, verification progress
  wallet-info      Show wallet name, balance, unconfirmed balance, tx count
  balance          Print the wallet balance
  new-address      Generate and print a new receiving address
  rpc              Execute an arbitrary Bitcoin Core RPC method
  help             Print this message or the help of the given subcommand(s)
```

### `blockchain-info`

```
$ cargo run -- blockchain-info
Chain:                 regtest
Blocks:                103
Headers:               103
Difficulty:            0.00000000046565423739069247
Verification progress: 1.000000
```

### `wallet-info`

```
$ cargo run -- wallet-info
Wallet name:         (default)
Balance:             150 BTC
Unconfirmed balance: 0 BTC
Immature balance:    5000 BTC
Transactions:        103
```

Notes:
- Bitcoin Core v30 removed `balance` / `unconfirmed_balance` from `getwalletinfo`; the CLI now composes `getwalletinfo` + `getbalances` transparently.
- `Immature balance` is coinbase output waiting for 100 confirmations. It is not part of the standard four fields the assessment asks for, but is useful context on regtest where mining constantly produces immature rewards.
- Polar's default wallet has an empty string as its name — the CLI displays this as `(default)` for readability.

### `balance`

```
$ cargo run -- balance
150 BTC
```

### `new-address`

```
$ cargo run -- new-address
bcrt1qtc3fm4xl6u3xcrdzrethan7w842jf66352knar
```

### `rpc <method> [params...]`

Passthrough for any Bitcoin Core RPC. Arguments are parsed as JSON where possible, otherwise as strings.

```
$ cargo run -- rpc getblockcount
103

$ cargo run -- rpc getblockhash 100
6b1d9fb7c20dd0649067d19f7156f773ae07184964e66e5b35818460e9733ae6

$ cargo run -- rpc getblock 6b1d9fb7c20dd0649067d19f7156f773ae07184964e66e5b35818460e9733ae6
{
  "hash": "6b1d9fb7c20dd0649067d19f7156f773ae07184964e66e5b35818460e9733ae6",
  "confirmations": 4,
  "height": 100,
  ...
}
```

---

## Error handling

The CLI never panics on user-facing failures. Every error surfaces through a typed `AppError` enum with a clear message.

### Invalid credentials

```
$ cargo run -- --rpc-password wrongpass blockchain-info
Error: invalid credentials — Bitcoin Core rejected the RPC auth (HTTP 401)
```

### Connection failure

```
$ cargo run -- --rpc-url http://127.0.0.1:9999 blockchain-info
Error: could not connect to Bitcoin Core at http://127.0.0.1:9999: ...
```

### Invalid RPC method

```
$ cargo run -- rpc totallynotamethod
Error: RPC error -32601: Method not found
```

### Invalid parameters

```
$ cargo run -- rpc getblockhash notanumber
Error: RPC error -3: Wrong type passed:
{
    "Position 1 (height)": "JSON value of type string is not of expected type number"
}
```

### Missing wallet

```
$ cargo run -- --wallet doesnotexist wallet-info
Error: wallet not found or not loaded on the node (looked for: 'doesnotexist')
```

---

## Project structure

```
src/
├── main.rs               Entry point — tracing init, parse CLI, dispatch to a command
├── cli.rs                clap definitions (subcommands + global config flags)
├── config.rs             Config::load — merges CLI > env > TOML > defaults
├── error.rs              AppError enum (thiserror)
├── rpc.rs                RpcClient with .call() / .call_wallet() / .call_raw()
└── commands/
    ├── mod.rs            Generic `rpc` passthrough
    ├── blockchain.rs     blockchain-info (typed BlockchainInfo struct)
    ├── wallet.rs         wallet-info + balance (typed WalletInfo / Balances structs)
    └── address.rs        new-address
```

The layout follows the suggestion in the assessment brief. Each command lives in its own module and depends only on `RpcClient` and the shared `AppError` type — no cross-module coupling.

---

## Design decisions

- **Async everywhere.** All RPC calls go through `reqwest`'s async client and a `#[tokio::main]` runtime. Cost is minimal, keeps the door open for concurrent calls in future commands.
- **Typed structs for the four named commands, raw JSON for the generic passthrough.** The typed structs (`BlockchainInfo`, `WalletInfo`, `Balances`) catch schema drift at deserialization time (that is exactly how we discovered the v30 change to `getwalletinfo`). The `rpc` subcommand deliberately returns `serde_json::Value` so users can call any method without a matching Rust struct.
- **Precedence: CLI flag > env var > TOML file > default.** This lets users pin credentials in a config file for daily use, override them with an env var in CI, and still shadow both from the shell for one-off calls.
- **`AppError` mapped from HTTP context.** `reqwest::Error::is_connect()` is used to distinguish "server unreachable" from other transport errors, so the user sees "could not connect" instead of an opaque HTTP failure. `401` triggers `InvalidCredentials`. Bitcoin Core error codes `-18` / `-19` are translated to `MissingWallet` before they reach the user.
- **`anyhow` at the binary edge, `thiserror` inside library code.** `main.rs` returns `anyhow::Result<()>` so any error type formats cleanly; the rest of the crate uses the typed `AppError` for explicit variant handling.
- **Empty default-wallet name is rendered as `(default)`.** Polar's default wallet is created with `walletname = ""`, which is easy to misread as a bug in the output. Substituting `(default)` avoids the confusion.
- **Wallet-scoped calls use `/wallet/<name>`.** Bitcoin Core requires the wallet name in the URL path when multiple wallets are loaded; the CLI does this automatically when `--wallet` is set.

---

## Bonus features implemented

From the assessment's optional list:

- [x] **Async implementation using Tokio.**
- [x] **Logging with `tracing`.** Set `RUST_LOG=debug` to see every RPC call.
- [x] **Reusable RPC client abstraction** (`RpcClient` in `src/rpc.rs`).
- [x] **Configuration file support** (`--config <path>`, TOML format).
- [x] **Support for multiple wallets** (`--wallet <name>` flag, wallet-scoped URL routing).
- [x] **Unit tests** — run with `cargo test`. Covers config precedence (CLI > file > defaults), TOML parsing, and wallet-URL routing.
- [x] **Pretty terminal output** — colored, TTY-aware output via `owo-colors`. Colors are stripped when piped/redirected, and can be disabled globally with `NO_COLOR=1`.
