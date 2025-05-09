FROM rust:1.76-slim as builder

WORKDIR /usr/src/ollama-agent
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release && \
    strip /usr/src/ollama-agent/target/release/ollama-agent

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ollama-agent/target/release/ollama-agent /usr/local/bin/

# Create a non-root user to run the application
RUN useradd -m -s /bin/bash ollama
USER ollama

EXPOSE 11434

# Set up environment variables with defaults
ENV RUST_LOG=info
ENV OLLAMA_PROXY_PORT=11434
ENV OLLAMA_REMOTE_URL=https://api.ollama.ai

ENTRYPOINT ["/usr/local/bin/ollama-agent", "--local-addr", "0.0.0.0:${OLLAMA_PROXY_PORT}", "--remote-url", "${OLLAMA_REMOTE_URL}"]