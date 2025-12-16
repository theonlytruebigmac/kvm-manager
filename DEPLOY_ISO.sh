#!/bin/bash
#
# Guest Agent ISO Deployment Script
#
# This script deploys the KVM Manager guest agent ISO to the libvirt images directory.
# Requires: sudo privileges
#
# Usage: sudo ./DEPLOY_ISO.sh
#

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "KVM Manager Guest Agent ISO Deployment"
echo "=========================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run with sudo${NC}"
    echo "Usage: sudo ./DEPLOY_ISO.sh"
    exit 1
fi

# Define paths
SOURCE_ISO="/home/fraziersystems/Documents/projects/kvm-manager/guest-agent/kvmmanager-guest-agent.iso"
TARGET_DIR="/var/lib/libvirt/images"
TARGET_ISO="$TARGET_DIR/kvmmanager-guest-agent.iso"

# Verify source ISO exists
echo -n "Checking source ISO... "
if [ ! -f "$SOURCE_ISO" ]; then
    echo -e "${RED}FAILED${NC}"
    echo "Error: Source ISO not found at: $SOURCE_ISO"
    exit 1
fi
echo -e "${GREEN}OK${NC}"

# Get source ISO info
SOURCE_SIZE=$(stat -f "%z" "$SOURCE_ISO" 2>/dev/null || stat -c "%s" "$SOURCE_ISO")
SOURCE_SIZE_MB=$(echo "scale=2; $SOURCE_SIZE / 1024 / 1024" | bc)
echo "  Size: ${SOURCE_SIZE_MB} MB"

# Verify target directory exists
echo -n "Checking target directory... "
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${RED}FAILED${NC}"
    echo "Error: Target directory not found: $TARGET_DIR"
    echo "Is libvirtd installed?"
    exit 1
fi
echo -e "${GREEN}OK${NC}"

# Check if ISO already exists
if [ -f "$TARGET_ISO" ]; then
    echo -e "${YELLOW}Warning: ISO already exists at target location${NC}"
    echo -n "Overwrite? (y/N): "
    read -r RESPONSE
    if [[ ! "$RESPONSE" =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled."
        exit 0
    fi
fi

# Copy ISO
echo -n "Copying ISO to $TARGET_DIR... "
if cp "$SOURCE_ISO" "$TARGET_ISO"; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Error: Failed to copy ISO"
    exit 1
fi

# Set permissions
echo -n "Setting permissions (644)... "
if chmod 644 "$TARGET_ISO"; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Warning: Failed to set permissions, but ISO is deployed"
fi

# Verify deployment
echo -n "Verifying deployment... "
if [ -f "$TARGET_ISO" ]; then
    TARGET_SIZE=$(stat -f "%z" "$TARGET_ISO" 2>/dev/null || stat -c "%s" "$TARGET_ISO")
    if [ "$SOURCE_SIZE" -eq "$TARGET_SIZE" ]; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${YELLOW}WARNING${NC}"
        echo "  File sizes don't match - deployment may be corrupted"
        exit 1
    fi
else
    echo -e "${RED}FAILED${NC}"
    echo "Error: ISO not found at target location after copy"
    exit 1
fi

# Show final status
echo ""
echo "=========================================="
echo -e "${GREEN}Deployment Successful!${NC}"
echo "=========================================="
echo ""
echo "ISO Location: $TARGET_ISO"
echo "Permissions: $(stat -f "%Sp" "$TARGET_ISO" 2>/dev/null || stat -c "%a" "$TARGET_ISO")"
echo "Size: ${SOURCE_SIZE_MB} MB"
echo ""
echo "Next Steps:"
echo "1. Create a test VM through KVM Manager UI"
echo "2. Click 'Mount Agent ISO' in the Guest Information card"
echo "3. Install agent inside VM using installation scripts"
echo "4. Follow testing checklist in guest-agent/DEPLOYMENT_CHECKLIST.md"
echo ""
echo "For installation instructions, see:"
echo "  guest-agent/INSTALL.md"
echo ""
