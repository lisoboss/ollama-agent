# Ollama Agent LaunchAgent Deployment Guide for macOS

This guide explains how to set up Ollama Agent as a LaunchAgent on macOS to run automatically at startup and restart if it crashes.

## Installation

1. **Build and install Ollama Agent**

   First, build and install the Ollama Agent:
   ```bash
   cargo build --release
   sudo cp target/release/ollama-agent /usr/local/bin/
   ```

   If you want keychain support:
   ```bash
   cargo build --release --features keychain
   sudo cp target/release/ollama-agent /usr/local/bin/
   ```

2. **Configure the LaunchAgent**

   Copy the LaunchAgent plist file to your user LaunchAgents directory:
   ```bash
   mkdir -p ~/Library/LaunchAgents
   cp deploy/launchd/com.user.ollama-agent.plist ~/Library/LaunchAgents/
   ```

3. **Customize settings (optional)**

   Edit the plist file to customize settings:
   ```bash
   nano ~/Library/LaunchAgents/com.user.ollama-agent.plist
   ```

   - Change the remote URL if needed
   - Uncomment and add your API key
   - Adjust the local binding address and port

4. **Load the LaunchAgent**

   Load the service with:
   ```bash
   launchctl load ~/Library/LaunchAgents/com.user.ollama-agent.plist
   ```

   Start the service:
   ```bash
   launchctl start com.user.ollama-agent
   ```

## Managing the Service

### Check Status

To verify the service is running:
```bash
launchctl list | grep ollama-agent
```

### View Logs

To check the logs:
```bash
cat /tmp/ollama-agent.out
cat /tmp/ollama-agent.err
```

### Restart the Service

To restart the service:
```bash
launchctl stop com.user.ollama-agent
launchctl start com.user.ollama-agent
```

### Disable the Service

To unload the service temporarily:
```bash
launchctl unload ~/Library/LaunchAgents/com.user.ollama-agent.plist
```

### Permanently Remove

To permanently remove the service:
```bash
launchctl unload ~/Library/LaunchAgents/com.user.ollama-agent.plist
rm ~/Library/LaunchAgents/com.user.ollama-agent.plist
```

## Troubleshooting

If the service fails to start:

1. Check the error logs:
   ```bash
   cat /tmp/ollama-agent.err
   ```

2. Verify the executable is at the correct path:
   ```bash
   ls -l /usr/local/bin/ollama-agent
   ```

3. Ensure the plist file has correct permissions:
   ```bash
   chmod 644 ~/Library/LaunchAgents/com.user.ollama-agent.plist
   ```

4. Try running the agent manually to check for issues:
   ```bash
   /usr/local/bin/ollama-agent --local-addr 127.0.0.1:11434
   ```