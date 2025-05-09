# Ollama Agent Docker Deployment Guide

This guide explains how to deploy Ollama Agent using Docker.

## Prerequisites

- Docker installed on your system
- Docker Compose (optional, but recommended)

## Quick Start

### Using Docker Compose (Recommended)

1. Navigate to the deploy/docker directory:
   ```bash
   cd deploy/docker
   ```

2. Edit the environment variables in `docker-compose.yml` if needed:
   - `OLLAMA_REMOTE_URL`: The remote Ollama API URL
   - `OLLAMA_API_KEY`: Your API key for authentication (optional)

3. Start the container:
   ```bash
   docker-compose up -d
   ```

4. Check the logs:
   ```bash
   docker-compose logs -f
   ```

### Using Docker Directly

1. Build the Docker image:
   ```bash
   docker build -t ollama-agent .
   ```

2. Run the container:
   ```bash
   docker run -d --name ollama-agent \
     -p 11434:11434 \
     -e OLLAMA_REMOTE_URL=https://api.ollama.ai \
     -e OLLAMA_API_KEY=your_api_key_here \
     ollama-agent
   ```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OLLAMA_PROXY_PORT` | Port to expose the proxy on | 11434 |
| `OLLAMA_REMOTE_URL` | Remote Ollama API URL | https://api.ollama.ai |
| `OLLAMA_API_KEY` | API key for authentication | (empty) |
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | info |

## Updating

To update to a new version:

```bash
# Using Docker Compose
cd deploy/docker
docker-compose pull
docker-compose up -d

# Using Docker directly
docker pull yourusername/ollama-agent:latest
docker stop ollama-agent
docker rm ollama-agent
# Then run the container again with the same parameters
```

## Security Considerations

- The container runs as a non-root user for security
- API keys are passed as environment variables
- For production use, consider using Docker secrets or a secure environment variable management solution

## Troubleshooting

If you encounter issues:

1. Check the container logs:
   ```bash
   docker logs ollama-agent
   ```

2. Verify connectivity:
   ```bash
   curl http://localhost:11434/api/tags
   ```

3. If the container won't start, check for port conflicts:
   ```bash
   netstat -tuln | grep 11434
   ```