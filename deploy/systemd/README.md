# Ollama Agent Systemd Deployment Guide for Linux

This guide explains how to set up Ollama Agent as a systemd service on Linux to run automatically at startup and restart if it crashes.

## Installation Options

You can install Ollama Agent as either:
1. A system-wide service (requires root/sudo)
2. A user-level service (runs under your user account)

## Automatic Installation (Recommended)

We provide an installation script that handles all the setup steps:

```bash
# Make the install script executable
chmod +x deploy/systemd/install.sh

# For user-level installation (recommended for most users)
./deploy/systemd/install.sh

# For system-wide installation (requires sudo)
sudo ./deploy/systemd/install.sh --system

# With custom options
./deploy/systemd/install.sh --api-key your_key_here --remote https://your-server.com
```

Run `./deploy/systemd/install.sh --help` to see all available options.

## Manual Installation

### System-wide Installation

1. **Build and install Ollama Agent**

   ```bash
   cargo build --release
   sudo cp target/release/ollama-agent /usr/local/bin/
   sudo chmod 755 /usr/local/bin/ollama-agent
   ```

2. **Create a dedicated user (optional but recommended)**

   ```bash
   sudo useradd --system --create-home --shell /sbin/nologin ollama
   ```

3. **Configure the service**

   Copy the service file to the systemd directory:
   ```bash
   sudo cp deploy/systemd/ollama-agent.service /etc/systemd/system/
   ```

   Edit the service file to customize settings:
   ```bash
   sudo nano /etc/systemd/system/ollama-agent.service
   ```

4. **Enable and start the service**

   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable ollama-agent
   sudo systemctl start ollama-agent
   ```

### User-level Installation

1. **Build and install Ollama Agent in your home directory**

   ```bash
   cargo build --release
   mkdir -p ~/.local/bin
   cp target/release/ollama-agent ~/.local/bin/
   chmod 755 ~/.local/bin/ollama-agent
   ```

2. **Configure the service**

   Create the systemd user directory:
   ```bash
   mkdir -p ~/.config/systemd/user
   ```

   Copy the service file:
   ```bash
   cp deploy/systemd/ollama-agent-user.service ~/.config/systemd/user/ollama-agent.service
   ```

   Edit the service file if needed:
   ```bash
   nano ~/.config/systemd/user/ollama-agent.service
   ```

3. **Enable and start the service**

   ```bash
   systemctl --user daemon-reload
   systemctl --user enable ollama-agent
   systemctl --user start ollama-agent
   ```

## Managing the Service

### System-wide Service Commands

```bash
# Check status
sudo systemctl status ollama-agent

# View logs
sudo journalctl -u ollama-agent

# Restart service
sudo systemctl restart ollama-agent

# Stop service
sudo systemctl stop ollama-agent

# Disable service
sudo systemctl disable ollama-agent
```

### User-level Service Commands

```bash
# Check status
systemctl --user status ollama-agent

# View logs
journalctl --user-unit ollama-agent

# Restart service
systemctl --user restart ollama-agent

# Stop service
systemctl --user stop ollama-agent

# Disable service
systemctl --user disable ollama-agent
```

## Customizing the Service

The systemd service files support several customization options:

- Local address/port to listen on
- Remote Ollama API URL
- API key for authentication
- Log verbosity

Edit the service file and modify the `ExecStart` line and environment variables as needed.

## Troubleshooting

If the service fails to start:

1. Check the service status:
   ```bash
   sudo systemctl status ollama-agent
   ```

2. Check detailed logs:
   ```bash
   sudo journalctl -u ollama-agent -n 50
   ```

3. Verify permissions:
   ```bash
   ls -la /usr/local/bin/ollama-agent
   ```

4. Try running manually to check for issues:
   ```bash
   /usr/local/bin/ollama-agent --local-addr 127.0.0.1:11434
   ```

5. Common fixes:
   - Ensure the binary exists at the path specified in the service file
   - Check that all required directories exist with correct permissions
   - Verify there are no port conflicts

## Security Considerations

The systemd service file includes several security-enhancing options:
- `NoNewPrivileges=true`: Prevents privilege escalation
- `ProtectSystem=full`: Restricts write access to system directories
- `PrivateTmp=true`: Provides a private /tmp directory

For even stronger security, consider additional sandboxing options in the systemd service file.