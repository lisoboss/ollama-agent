#!/bin/bash
# Install script for Ollama Agent systemd service

set -e

# Define colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
SYSTEM_WIDE=0
API_KEY=""
REMOTE_URL="https://api.ollama.ai"
LOCAL_ADDR="127.0.0.1:11434"
BINARY_PATH=""

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -s, --system       Install as a system service (requires sudo)"
    echo "  -k, --api-key KEY  Set API key for authentication"
    echo "  -r, --remote URL   Set remote Ollama API URL (default: $REMOTE_URL)"
    echo "  -l, --local ADDR   Set local address to bind to (default: $LOCAL_ADDR)"
    echo "  -b, --binary PATH  Path to the ollama-agent binary (default: auto-detect)"
    echo "  -h, --help         Show this help message"
    echo
    echo "Example:"
    echo "  $0 --api-key your_key_here --remote https://your-ollama-server.com"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--system)
            SYSTEM_WIDE=1
            shift
            ;;
        -k|--api-key)
            API_KEY="$2"
            shift 2
            ;;
        -r|--remote)
            REMOTE_URL="$2"
            shift 2
            ;;
        -l|--local)
            LOCAL_ADDR="$2"
            shift 2
            ;;
        -b|--binary)
            BINARY_PATH="$2"
            shift 2
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            print_usage
            exit 1
            ;;
    esac
done

# Auto-detect binary location if not specified
if [ -z "$BINARY_PATH" ]; then
    if [ -f "/usr/local/bin/ollama-agent" ]; then
        BINARY_PATH="/usr/local/bin/ollama-agent"
    elif [ -f "$HOME/.local/bin/ollama-agent" ]; then
        BINARY_PATH="$HOME/.local/bin/ollama-agent"
    elif [ -f "./target/release/ollama-agent" ]; then
        BINARY_PATH="$(pwd)/target/release/ollama-agent"
    else
        echo -e "${RED}Error: Cannot find ollama-agent binary.${NC}"
        echo "Please specify with --binary option or install it first."
        exit 1
    fi
fi

echo -e "${GREEN}Using binary: ${BINARY_PATH}${NC}"

# Check if we're running as root for system-wide installation
if [ "$SYSTEM_WIDE" -eq 1 ] && [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}Error: System-wide installation requires root privileges.${NC}"
    echo "Please run with sudo or switch to user installation with --user."
    exit 1
fi

# Prepare the service file with the correct paths and settings
prepare_service_file() {
    local service_file="$1"
    local template="$2"
    
    # Create a copy of the template
    cp "$template" "$service_file"
    
    # Replace placeholders
    sed -i "s|ExecStart=.*|ExecStart=${BINARY_PATH} --local-addr ${LOCAL_ADDR} --remote-url ${REMOTE_URL}|" "$service_file"
    
    # Add API key if provided
    if [ -n "$API_KEY" ]; then
        sed -i "s|#Environment=OLLAMA_API_KEY=.*|Environment=OLLAMA_API_KEY=${API_KEY}|" "$service_file"
    fi
}

# Install system-wide service
install_system_service() {
    echo -e "${YELLOW}Installing system-wide service...${NC}"
    
    # Check if the ollama user exists, create if it doesn't
    if ! id -u ollama >/dev/null 2>&1; then
        echo "Creating ollama user..."
        useradd --system --create-home --shell /sbin/nologin ollama
    fi
    
    # Copy binary if necessary
    if [ "$BINARY_PATH" != "/usr/local/bin/ollama-agent" ]; then
        echo "Copying binary to /usr/local/bin/..."
        cp "$BINARY_PATH" /usr/local/bin/ollama-agent
        chmod 755 /usr/local/bin/ollama-agent
        BINARY_PATH="/usr/local/bin/ollama-agent"
    fi
    
    # Prepare and install service file
    prepare_service_file "/tmp/ollama-agent.service" "$(dirname "$0")/ollama-agent.service"
    cp /tmp/ollama-agent.service /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable ollama-agent.service
    systemctl start ollama-agent.service
    
    echo -e "${GREEN}System service installed and started!${NC}"
    echo "To check status: sudo systemctl status ollama-agent"
    echo "To view logs: sudo journalctl -u ollama-agent"
}

# Install user service
install_user_service() {
    echo -e "${YELLOW}Installing user service...${NC}"
    
    # Create ~/.local/bin if it doesn't exist
    mkdir -p ~/.local/bin
    
    # Copy binary if necessary
    if [ "$BINARY_PATH" != "$HOME/.local/bin/ollama-agent" ]; then
        echo "Copying binary to ~/.local/bin/..."
        cp "$BINARY_PATH" ~/.local/bin/ollama-agent
        chmod 755 ~/.local/bin/ollama-agent
        BINARY_PATH="$HOME/.local/bin/ollama-agent"
    fi
    
    # Create systemd user directory if it doesn't exist
    mkdir -p ~/.config/systemd/user
    
    # Prepare and install service file
    prepare_service_file "/tmp/ollama-agent-user.service" "$(dirname "$0")/ollama-agent-user.service"
    cp /tmp/ollama-agent-user.service ~/.config/systemd/user/ollama-agent.service
    systemctl --user daemon-reload
    systemctl --user enable ollama-agent.service
    systemctl --user start ollama-agent.service
    
    echo -e "${GREEN}User service installed and started!${NC}"
    echo "To check status: systemctl --user status ollama-agent"
    echo "To view logs: journalctl --user-unit ollama-agent"
}

# Main installation logic
if [ "$SYSTEM_WIDE" -eq 1 ]; then
    install_system_service
else
    install_user_service
fi

echo -e "\n${GREEN}Installation complete!${NC}"
echo -e "Ollama Agent is now configured to proxy to ${YELLOW}${REMOTE_URL}${NC}"
echo -e "and is listening on ${YELLOW}${LOCAL_ADDR}${NC}"