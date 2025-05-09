# Ollama Agent Deployment Guide

This directory contains deployment configurations for various environments. Ollama Agent can be deployed in multiple ways depending on your operating system and requirements.

## Available Deployment Options

### Docker Deployment

[Docker deployment instructions](docker/README.md)

- Containerized deployment
- Environment variable configuration
- Easy updates via Docker Compose
- Cross-platform compatibility
- Isolation from host system

**Ideal for:** Production environments, CI/CD pipelines, cross-platform deployment

### macOS LaunchAgent

[macOS LaunchAgent instructions](launchd/README.md)

- Automatic startup at login
- Built-in restart capabilities
- Low-level system integration
- Keychain integration support

**Ideal for:** macOS users, desktop deployments, personal use

### Linux Systemd Service

[Systemd service instructions](systemd/README.md)

- System-wide or user-level service
- Automatic startup and restart
- Detailed logging via journald
- Security hardening options

**Ideal for:** Linux servers, desktop Linux, production environments

## Quick Comparison

| Feature | Docker | LaunchAgent | Systemd |
|---------|--------|-------------|---------|
| Cross-platform | ✅ | ❌ | ❌ |
| Automatic restart | ✅ | ✅ | ✅ |
| Isolation | High | Low | Medium |
| Setup complexity | Medium | Low | Low |
| Performance impact | Medium | Low | Low |
| Updates | Easy | Manual | Manual |
| Keychain support | Limited | ✅ | N/A |

## General Deployment Recommendations

1. **Personal Use**: 
   - On macOS: LaunchAgent
   - On Linux: User-level systemd service
   - On Windows: Docker

2. **Server Deployment**:
   - Docker for cross-platform consistency
   - Systemd for Linux-specific optimizations

3. **Development Environment**:
   - Run directly without deployment configuration
   - Use the provided installation scripts for quick setup

## Configuration Best Practices

Regardless of deployment method, consider these best practices:

- Store API keys securely (use keychain on macOS when possible)
- Limit access to the service on shared systems
- Configure proper logging for troubleshooting
- Set up health monitoring for production deployments

## Getting Help

If you encounter issues with any deployment method, please:

1. Check the specific README for your deployment method
2. Look at the application logs for error messages
3. Run the application manually to identify issues
4. Check the GitHub issues for similar problems