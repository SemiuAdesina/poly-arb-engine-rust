# PolyArb Engine

A professional, modular arbitrage trading bot for Polymarket with Telegram control interface.

## Overview

The PolyArb Engine is a Rust-based trading bot that scans for arbitrage opportunities on Polymarket and executes trades automatically. It features:

- Modular Architecture: Clean separation of concerns (config, types, clients, engine, execution, utils)
- Telegram Bot Interface: Easy-to-use control panel with inline buttons
- 24/7 Operation: Designed to run continuously on a VPS
- Real-time Monitoring: Live orderbook feeds and balance tracking
- Cloudflare Bypass: Uses FlareSolverr with residential proxies to bypass Cloudflare protection
- Varied Profit Thresholds: Randomly selects profit thresholds (0.8%, 1.5%, 2.0%, 2.5%) per order

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- Ubuntu/Debian VPS (for production)
- Polymarket API credentials
- Telegram Bot Token (from @BotFather)
- Docker (for FlareSolverr)
- Residential Proxy Service (for Cloudflare bypass)

## Project Structure

```
poly-arb-engine/
├── Cargo.toml              # Dependencies
├── .env                    # Configuration (create from .env.example)
├── poly-bot.service       # Systemd service file
├── src/
│   ├── main.rs            # Entry point
│   ├── config/            # Settings & environment loading
│   ├── types/             # Data models
│   ├── clients/           # Network layer (Polymarket, Polygon, Telegram)
│   ├── engine/            # Logic layer (discovery, strategy, risk)
│   ├── execution/         # Action layer (signing, trading)
│   ├── telegram/          # Telegram bot
│   └── utils/             # Helpers
```

## Installation

### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
```

### Step 2: Clone and Build

```bash
cd poly-arb-engine
cargo build --release
```

### Step 3: Install Docker (for FlareSolverr)

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
docker --version
```

## Configuration

### Environment Variables

Create a `.env` file in the project root with the following variables:

#### Required Variables

```env
# Blockchain Configuration
RPC_URL=https://polygon-rpc.com
WS_URL=wss://clob.polymarket.com
PRIVATE_KEY=0x_your_private_key_here

# Polymarket API Credentials (L2 Authentication)
POLY_API_KEY=your_api_key_here
POLY_API_SECRET=your_api_secret_here
POLY_PASSPHRASE=your_passphrase_here

# Telegram Bot Configuration
TELEGRAM_BOT_TOKEN=your_telegram_bot_token_here
TELEGRAM_CHAT_ID=your_chat_id_here
```

#### Optional Variables

```env
# Cloudflare Bypass (MANDATORY for VPS deployment)
FLARESOLVERR_URL=http://localhost:8191
FLARESOLVERR_PROXY=http://username:password@residential-proxy.com:8080

# Cloudflare Cookie (alternative method, less reliable)
CLOUDFLARE_COOKIE="cookie1=value1; cookie2=value2"

# Safety Settings
MAX_USDC_PER_MARKET=100.0
MAX_WALLET_PERCENT_PER_ORDER=0.1
ONE_MARKET_AT_A_TIME=false
MIN_PROFIT_PCT=0.2

# Trading Mode
TEST_MODE=false
DRY_RUN=false
MANUAL_TRADING_ENABLED=true
```

### Getting Polymarket API Credentials

#### Method 1: Manual Extraction (Recommended)

1. Open Polymarket in your browser and log in
2. Open Developer Tools (F12)
3. Go to Network tab
4. Place a small test trade or view your portfolio
5. Find requests to `clob.polymarket.com`
6. In the request headers, find `x-api-key` and copy the value
7. Add to `.env` as `POLY_API_KEY`

Note: Manual extraction only gives you the API key. You will need to obtain the secret and passphrase through Polymarket's official API or by using their official client libraries (py-clob-client for Python or clob-client for TypeScript).

### Getting Telegram Bot Token

1. Open Telegram and search for @BotFather
2. Send `/newbot` and follow instructions
3. Save the bot token provided
4. Add token to `.env` as `TELEGRAM_BOT_TOKEN`

To get your Chat ID:
1. Send `/start` to your bot
2. Run: `curl "https://api.telegram.org/botYOUR_TOKEN/getUpdates"`
3. Find the `chat.id` in the response
4. Add to `.env` as `TELEGRAM_CHAT_ID`

## Cloudflare Bypass Setup

Cloudflare blocks datacenter IPs (VPS, AWS, DigitalOcean, etc.) with hard CAPTCHAs. Your VPS IP is flagged as "Low Reputation" while residential IPs are "High Reputation."

### Solution: Residential Proxy + FlareSolverr

You MUST use a residential proxy to make your VPS appear as a home internet connection. Datacenter proxies will still be blocked.

#### Step 1: Get a Residential Proxy Service

Recommended providers:
- Bright Data (brightdata.com) - Premium, ~$500/month
- Oxylabs (oxylabs.io) - Premium, ~$300/month
- Smartproxy (smartproxy.com) - Budget-friendly, ~$150/month

For trading bots, you typically need ~1-5GB/month depending on trade frequency.

Get your proxy credentials in the format:
```
http://username:password@proxy-host:port
```

#### Step 2: Install and Configure FlareSolverr

FlareSolverr is a proxy server that runs a headless Chrome browser to automatically solve Cloudflare challenges.

```bash
# Run FlareSolverr in Docker
docker run -d \
  --name=flaresolverr \
  -p 8191:8191 \
  -e LOG_LEVEL=info \
  --restart unless-stopped \
  ghcr.io/flaresolverr/flaresolverr:latest

# Verify it's running
docker ps | grep flaresolverr

# Check logs
docker logs flaresolverr
```

Test FlareSolverr:
```bash
curl -X POST http://localhost:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{
    "cmd": "request.get",
    "url": "https://clob.polymarket.com",
    "maxTimeout": 60000
  }'
```

#### Step 3: Configure Proxy in .env

Add to your `.env` file:

```env
FLARESOLVERR_URL=http://localhost:8191
FLARESOLVERR_PROXY=http://username:password@residential-proxy.com:8080
```

#### Step 4: Alternative - Configure Proxy in Docker

If you prefer to configure the proxy at the Docker level:

```bash
docker stop flaresolverr
docker rm flaresolverr

docker run -d --name=flaresolverr -p 8191:8191 \
  -e PROXY="http://username:password@residential-proxy.com:8080" \
  -e LOG_LEVEL=info \
  --restart unless-stopped \
  ghcr.io/flaresolverr/flaresolverr:latest
```

#### How It Works

1. Bot makes POST request → Routes through FlareSolverr (if `FLARESOLVERR_URL` is set)
2. FlareSolverr → Uses residential proxy → Opens headless Chrome → Solves Cloudflare challenge
3. Response → Returns 200 OK with order ID

#### Testing

After setup, watch the logs. You should see:
- "Routing request through FlareSolverr (may take 10-30 seconds)..."
- "Using proxy: http://..."
- "Order placed successfully! Order ID: ..."

If you still see `403 Forbidden`:
1. Check proxy credentials are correct
2. Verify proxy is residential (not datacenter)
3. Try a different proxy IP from your provider
4. Contact proxy provider support

### Alternative: Cloudflare Cookie Method (Less Reliable)

If you cannot use a residential proxy, you can try extracting a Cloudflare cookie manually using browser developer tools:

1. Open Polymarket in your browser
2. Open Developer Tools (F12)
3. Go to Application/Storage tab
4. Find Cookies for `polymarket.com`
5. Copy the `cf_clearance` cookie value
6. Add to `.env` as: `CLOUDFLARE_COOKIE="cf_clearance=value; other_cookies=values"`

Note: Cookies expire after a few hours/days and must be refreshed. The residential proxy method is more reliable.

## Running the Bot

### Local Development

```bash
# Run in development mode
cargo run

# Run release binary
./target/release/poly-arb-engine
```

### Production Deployment on VPS

#### Step 1: Upload Files to VPS

```bash
# Upload binary
scp ./target/release/poly-arb-engine root@your_vps_ip:/root/poly-arb-engine/

# Upload .env file
scp ./.env root@your_vps_ip:/root/poly-arb-engine/

# Upload service file
scp ./poly-bot.service root@your_vps_ip:/root/poly-arb-engine/
```

#### Step 2: Setup on VPS

```bash
ssh root@your_vps_ip
cd /root/poly-arb-engine

# Make binary executable
chmod +x poly-arb-engine

# Verify .env file exists
ls -la .env
```

#### Step 3: Configure Systemd Service

```bash
# Copy service file
sudo cp poly-bot.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable poly-bot

# Start the service
sudo systemctl start poly-bot
```

#### Step 4: Verify

```bash
# Check status
sudo systemctl status poly-bot

# View logs
sudo journalctl -u poly-bot -f
```

## Trading Modes

### DRY_RUN Mode

Set `DRY_RUN=true` in `.env` to prevent any orders from being placed. Useful for testing:

```env
DRY_RUN=true
```

### TEST_MODE

Set `TEST_MODE=true` to place real orders but with deep limits (95% discount) that won't fill immediately:

```env
TEST_MODE=true
```

This allows you to verify order placement on the Polymarket website without risking immediate fills.

### Production Mode

Leave both `DRY_RUN` and `TEST_MODE` unset or set to `false` for live trading.

## Profit Thresholds

The bot uses varied profit thresholds to make behavior less predictable. For each order evaluation, the bot randomly selects from:

- 0.8%
- 1.5%
- 2.0%
- 2.5%

Orders are only placed if the expected profit meets or exceeds the randomly selected threshold.

## Safety Settings

Configure risk management in `.env`:

```env
# Maximum USDC to risk per market
MAX_USDC_PER_MARKET=100.0

# Maximum percentage of wallet to use per order (0.0-1.0)
MAX_WALLET_PERCENT_PER_ORDER=0.1

# Only trade one market at a time
ONE_MARKET_AT_A_TIME=false
```

## Service Management

### Common Commands

```bash
# Start bot
sudo systemctl start poly-bot

# Stop bot
sudo systemctl stop poly-bot

# Restart bot
sudo systemctl restart poly-bot

# Check status
sudo systemctl status poly-bot

# Enable on boot
sudo systemctl enable poly-bot

# Disable on boot
sudo systemctl disable poly-bot
```

### Viewing Logs

```bash
# View live logs
sudo journalctl -u poly-bot -f

# View recent logs (last 100 lines)
sudo journalctl -u poly-bot -n 100

# View logs from today
sudo journalctl -u poly-bot --since today
```

## Troubleshooting

### Bot Not Responding to Commands

1. Clear message queue: Have user stop/restart bot in Telegram
2. Check for multiple instances: `ps aux | grep poly-arb-engine`
3. Verify token: Check token in `.env` matches BotFather
4. Check privacy mode: Disable in BotFather if enabled
5. Check logs: `sudo journalctl -u poly-bot -f` for errors

### Cloudflare 403 Forbidden Errors

1. Verify FlareSolverr is running: `docker ps | grep flaresolverr`
2. Check proxy credentials: Verify `FLARESOLVERR_PROXY` in `.env` is correct
3. Verify proxy is residential: Datacenter proxies will still be blocked
4. Test FlareSolverr directly: Use the curl command in the FlareSolverr setup section
5. Check FlareSolverr logs: `docker logs flaresolverr`

### Service Won't Start

1. Check binary: `ls -lh /root/poly-arb-engine/poly-arb-engine`
2. Check permissions: `chmod +x /root/poly-arb-engine/poly-arb-engine`
3. Check logs: `sudo journalctl -u poly-bot -n 50`
4. Test manually: Run binary directly to see errors
5. Verify paths: Ensure all paths in service file are correct

### API Authentication Errors

1. Verify API credentials: Check `POLY_API_KEY`, `POLY_API_SECRET`, `POLY_PASSPHRASE` in `.env`
2. Regenerate credentials: Use Polymarket's official client libraries or contact Polymarket support
3. Check private key: Ensure `PRIVATE_KEY` matches the wallet used to generate API credentials
4. Restart bot: `sudo systemctl restart poly-bot`

### Balance Check Failures

1. Verify RPC URL: Check `RPC_URL` in `.env` is correct
2. Verify private key: Ensure `PRIVATE_KEY` is correct (starts with 0x)
3. Check network: Ensure VPS can reach Polygon RPC endpoint
4. Check wallet: Verify wallet has funds on Polygon network

## Security Notes

- Never commit `.env` file - it's in `.gitignore`
- API keys can only trade, not withdraw (Polymarket restriction)
- Private key is secure - only used for signing transactions
- Telegram communication is encrypted
- Keep `.env` file permissions restricted: `chmod 600 .env`
- Use residential proxies from reputable providers
- Rotate API credentials periodically

## Architecture

The engine is designed to run on a **Residential IP** to bypass Cloudflare bot detection naturally, without needing expensive heavy-duty proxies.

```mermaid
graph TD
    %% Nodes
    User[Local Machine / Residential IP]
    
    subgraph "Rust Engine Core"
        Scanner[Market Scanner]
        Strategy[Strategy Module<br/>Negative Risk Fishing]
        Signer[EIP-712 Signer]
        Safety[Safety & Balance Checks]
    end
    
    subgraph "External Infrastructure"
        PolyAPI[Polymarket CLOB API<br/>Cloudflare Protected]
        Blockchain[Polygon RPC<br/>On-Chain Data]
    end

    %% Flow
    User -->|Starts| Scanner
    Scanner -->|Fetch Markets| PolyAPI
    PolyAPI -->|Market Data| Strategy
    
    Strategy -->|1. Identify Opportunity| Safety
    Safety -->|2. Verify Balance| Blockchain
    
    Safety -->|3. Approve Trade| Signer
    Signer -->|4. Sign Order Local Key| Signer
    Signer -->|5. Submit Signed Order| PolyAPI
    
    PolyAPI -->|6. Execution Report| Safety
    Safety -->|7. Auto-Redeem Winnings| Blockchain

    %% Styling
    style User fill:#d4f1f4,stroke:#333,stroke-width:2px
    style Signer fill:#ffcccb,stroke:#333,stroke-width:2px
    style PolyAPI fill:#fffbe7,stroke:#f66,stroke-width:2px,stroke-dasharray: 5 5
```

The bot follows a clean, modular architecture:

- **config/**: Configuration management and environment variables
- **types/**: Data structures for markets, orderbooks, responses
- **clients/**: External API connections (Polymarket, Polygon, Telegram)
- **engine/**: Core trading logic (discovery, strategy, risk)
- **execution/**: Trade execution and transaction signing
- **telegram/**: User interface and command handling
- **utils/**: Helper functions and logging

This structure makes it easy to:
- Change strategies without touching network code
- Swap API providers without changing logic
- Test components in isolation
- Maintain and extend functionality

## Systemd Service Configuration

The `poly-bot.service` file includes:

- **WorkingDirectory**: `/root/poly-arb-engine` (where .env file is located)
- **EnvironmentFile**: Loads variables from `.env` file
- **Restart**: Always (auto-restart on failure)
- **Logging**: All output to journalctl

Key settings:

```ini
[Service]
WorkingDirectory=/root/poly-arb-engine
EnvironmentFile=/root/poly-arb-engine/.env
ExecStart=/root/poly-arb-engine/poly-arb-engine
Restart=always
RestartSec=10
```

Important: The `EnvironmentFile` directive is critical - it ensures systemd loads all environment variables from `.env` before starting the process.

## Testing

### Test Telegram Token

```bash
curl "https://api.telegram.org/botYOUR_TOKEN/getMe"
```

### Test Bot Connection

```bash
curl -X POST "https://api.telegram.org/botYOUR_TOKEN/sendMessage" \
     -d chat_id=YOUR_CHAT_ID \
     -d text="Test message"
```

### Test FlareSolverr

```bash
curl -X POST http://localhost:8191/v1 \
  -H "Content-Type: application/json" \
  -d '{
    "cmd": "request.get",
    "url": "https://clob.polymarket.com",
    "maxTimeout": 60000
  }'
```

## Disclaimer

This software is for educational purposes only. Trading cryptocurrencies and binary options involves significant risk. The authors are not responsible for any financial losses incurred while using this bot. Review the code before running with real funds.

**Use at your own risk. Always test thoroughly in TEST_MODE or DRY_RUN mode before using real funds.**

## License

MIT License

Copyright (c) 2024 PolyArb Engine Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Support

For issues or questions, check the logs first:

```bash
sudo journalctl -u poly-bot -f
```

Most issues can be resolved by:
1. Checking service status
2. Verifying `.env` configuration
3. Reviewing logs for errors
4. Testing Telegram token validity
5. Verifying FlareSolverr and proxy setup
