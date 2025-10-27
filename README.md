# Celo Faucet

A minimal Rust-based faucet application that sends native tokens on custom EVM networks. Features a web UI and Telegram bot interface.

## Features

- **HTTP API**: RESTful endpoint for requesting tokens
- **Web UI**: Simple, beautiful web interface for requesting tokens
- **Telegram Bot**: Interactive bot for requesting tokens via Telegram
- **One-send-per-address**: Built-in protection against repeated requests
- **EVM Compatible**: Works with any EVM-compatible network

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Node.js 18+ (for building the web frontend)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd celo-faucet
```

2. Build the web frontend:
```bash
cd web
npm install
npm run build
cd ..
```

3. Build the Rust binary:
```bash
cargo build --release
```

## Configuration

Create a `.env` file in the project root with the following variables:

```bash
# Required
RPC_URL=https://your-rpc-endpoint.com
CHAIN_ID=44787
FAUCET_PRIVATE_KEY=0x...
FAUCET_AMOUNT_WEI=1000000000000000000

# Optional
BIND_ADDR=0.0.0.0:8080
STATE_PATH=./state
TELEGRAM_BOT_TOKEN=your_bot_token_from_botfather
TRACING_JSON=false
```

### Environment Variables

- `RPC_URL`: HTTP(S) JSON-RPC endpoint for your EVM network
- `CHAIN_ID`: Numeric chain ID of your network
- `FAUCET_PRIVATE_KEY`: Private key (0x-prefixed hex) of the faucet account
- `FAUCET_AMOUNT_WEI`: Amount to send per request in wei (as a decimal string)
- `BIND_ADDR`: Address and port to bind the HTTP server (default: `0.0.0.0:8080`)
- `STATE_PATH`: Path to the sled database for tracking addresses (default: `./state`)
- `TELEGRAM_BOT_TOKEN`: Optional Telegram bot token from [@BotFather](https://t.me/botfather)
- `TRACING_JSON`: Output logs in JSON format (default: `false`)

## Running

Start the faucet:

```bash
cargo run --release
```

Or run the compiled binary directly:

```bash
./target/release/celo-faucet
```

The faucet will:
- Start an HTTP server on the configured bind address
- Serve the web UI at `/`
- Listen for faucet requests at `/faucet`
- Start the Telegram bot (if token is configured)

## API Usage

### POST /faucet

Request tokens for an address.

**Request:**
```json
{
  "address": "0x..."
}
```

**Response (200 OK):**
```json
{
  "txHash": "0x..."
}
```

**Response (409 Conflict):**
```json
{
  "error": "already_sent"
}
```

**Response (400 Bad Request):**
```json
{
  "error": "Invalid address: ..."
}
```

### GET /healthz

Health check endpoint.

**Response:**
```json
{
  "status": "ok"
}
```

## Web UI

Access the web interface by navigating to `http://localhost:8080` (or your configured bind address) in a browser.

## Telegram Bot Setup

1. Create a bot with [@BotFather](https://t.me/botfather)
2. Copy the bot token
3. Add the token to your `.env` file as `TELEGRAM_BOT_TOKEN`
4. Restart the faucet
5. Users can interact with your bot:
   - Send `/start` to begin
   - Send their Ethereum address
   - Receive tokens and transaction hash

## Development

### Rebuild web frontend

```bash
cd web
npm run build
```

### Run in development mode with live logs

```bash
RUST_LOG=debug cargo run
```

### Code Structure

- `src/config.rs` - Environment configuration
- `src/store.rs` - Sled-based address tracking
- `src/jsonrpc.rs` - Minimal JSON-RPC client
- `src/tx.rs` - Transaction building and signing with Alloy
- `src/faucet.rs` - Core faucet service logic
- `src/http.rs` - Salvo HTTP server and routes
- `src/telegram.rs` - Teloxide Telegram bot
- `src/logging.rs` - Tracing initialization
- `src/main.rs` - Application entry point
- `web/` - TypeScript web frontend

## License

MIT

