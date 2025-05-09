# Ollama API Proxy

A lightweight HTTP proxy for remote Ollama API that optionally adds authentication headers to your requests. This tool is useful when you need to connect to a remote Ollama API that requires authentication.

## Features

- Proxies HTTP requests to a remote Ollama API endpoint
- Optionally adds authentication headers (Bearer token)
- Configurable local address and remote endpoint
- Support for streaming responses
- Graceful shutdown on Ctrl+C
- Optional macOS Keychain integration for securely storing API keys per remote URL
- Multiple deployment options (Docker, systemd, launchd)

## Installation

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- For macOS Keychain support: Xcode or Xcode Command Line Tools
- For deployment: Docker, systemd (Linux), or launchd (macOS) as needed

### Building from source

Clone the repository and build:

```bash
git clone https://github.com/yourusername/ollama-agent
cd ollama-agent
cargo build --release
```

The compiled binary will be available at `target/release/ollama-agent`.

For macOS Keychain support:

```bash
cargo build --release --features keychain
```

## Usage

Run the proxy with your API key (if authentication is required):

```bash
./ollama-agent --api-key your_api_key_here
```

Or set the API key via environment variable:

```bash
export OLLAMA_API_KEY=your_api_key_here
./ollama-agent
```

If your remote Ollama API doesn't require authentication, you can run without an API key:

```bash
./ollama-agent
```

### Command-line options

```
Usage: ollama-agent [OPTIONS]

Options:
  -l, --local-addr <LOCAL_ADDR>  Local address to bind to [default: 127.0.0.1:11434]
  -r, --remote-url <REMOTE_URL>  Remote Ollama API URL [default: https://api.ollama.ai]
  -a, --api-key <API_KEY>        API key for authentication (optional) [env: OLLAMA_API_KEY=]
      --save-key                 Save API key to macOS Keychain for the specified remote URL (requires keychain feature)
      --use-keychain             Use API key from macOS Keychain for the specified remote URL if not provided [default: true]
      --delete-key               Delete saved API key from macOS Keychain for the specified remote URL (requires keychain feature)
      --list-keys                List all remote URLs with saved API keys in macOS Keychain (requires keychain feature)
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples

Run with custom local address and remote URL:

```bash
./ollama-agent --local-addr 0.0.0.0:9000 --remote-url https://your-ollama-server.com
```

## Using with clients

Once the proxy is running, configure your Ollama clients to connect to the local proxy address instead of the remote server. For example:

```bash
# Instead of
ollama run llama2 --api-url https://remote-ollama-server.com

# Use
ollama run llama2 --api-url http://127.0.0.1:11434
```

### Example client

The repository includes an example client in Rust that demonstrates how to use the proxy:

```bash
# Run the example client (make sure the proxy is running)
cargo run --example client

# Set a custom proxy URL
PROXY_URL=http://127.0.0.1:11434 cargo run --example client
```

The example client tests connection to the models endpoint and runs a simple text generation.

## Advanced Usage

### Environment Variables

- `OLLAMA_API_KEY`: Set your API key without passing it on the command line (optional)
- `RUST_LOG`: Control log level (e.g., `info`, `debug`, `trace`)

Examples:
```bash
# With API key
RUST_LOG=debug OLLAMA_API_KEY=your_key ./ollama-agent

# Without API key
RUST_LOG=debug ./ollama-agent
```

### macOS Keychain Integration

The proxy can optionally integrate with the macOS Keychain to securely store and retrieve API keys. This feature is not compiled in by default and must be explicitly enabled at build time.

API keys are stored based on the remote URL, allowing you to use different API keys for different Ollama servers.

#### Building with Keychain Support

```bash
# Build with Keychain support
cargo build --release --features keychain
```

#### Using Keychain Features

Save your API key to the Keychain for a specific remote URL:
```bash
# Save API key for the default remote URL
./ollama-agent --api-key your_api_key --save-key

# Save API key for a custom remote URL
./ollama-agent --api-key your_api_key --remote-url https://your-ollama-server.com --save-key
```

Use a previously saved API key from Keychain:
```bash
# The --use-keychain flag is enabled by default
# Will use the API key for the default remote URL
./ollama-agent

# Use the API key for a specific remote URL
./ollama-agent --remote-url https://your-ollama-server.com
```

Delete a saved API key from Keychain:
```bash
# Delete key for the default remote URL
./ollama-agent --delete-key

# Delete key for a specific remote URL
./ollama-agent --remote-url https://your-ollama-server.com --delete-key
```

List all remote URLs with saved API keys:
```bash
./ollama-agent --list-keys
```

## Deployment Options

Ollama Agent can be deployed in various ways depending on your environment:

### Docker Deployment

For containerized deployment with Docker:

```bash
# Using docker-compose
cd deploy/docker
docker-compose up -d

# Or build and run directly
docker build -t ollama-agent .
docker run -d -p 11434:11434 ollama-agent
```

See [Docker deployment guide](deploy/docker/README.md) for more details.

### Linux Systemd Service

For Linux systems using systemd:

```bash
# Automatic installation (user-level)
./deploy/systemd/install.sh

# Or system-wide with custom options
sudo ./deploy/systemd/install.sh --system --api-key your_key --remote https://your-server.com
```

See [Systemd deployment guide](deploy/systemd/README.md) for more details.

### macOS LaunchAgent

For macOS with automatic startup:

```bash
# Copy LaunchAgent plist to your user directory
mkdir -p ~/Library/LaunchAgents
cp deploy/launchd/com.user.ollama-agent.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.user.ollama-agent.plist
```

See [LaunchAgent deployment guide](deploy/launchd/README.md) for more details.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.