#!/bin/bash
# Script to create a test VM with virtio-serial channel for guest agent testing

set -e

# Configuration
VM_NAME="kvmmanager-test"
ISO_PATH="${1:-}"
DISK_SIZE="20G"
MEMORY="2048"
VCPUS="2"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== KVM Manager Guest Agent Test VM Setup ===${NC}\n"

# Check if ISO path provided
if [ -z "$ISO_PATH" ]; then
    echo -e "${YELLOW}No ISO path provided. Please provide a Linux ISO path:${NC}"
    echo "Usage: $0 /path/to/linux.iso"
    echo ""
    echo "You can download a Linux ISO from:"
    echo "  - Ubuntu: https://ubuntu.com/download/desktop"
    echo "  - Fedora: https://fedoraproject.org/workstation/download"
    echo "  - Debian: https://www.debian.org/distrib/netinst"
    exit 1
fi

# Check if ISO exists
if [ ! -f "$ISO_PATH" ]; then
    echo -e "${RED}Error: ISO file not found: $ISO_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}Creating VM: $VM_NAME${NC}"
echo "  ISO: $ISO_PATH"
echo "  Disk: $DISK_SIZE"
echo "  Memory: ${MEMORY}MB"
echo "  vCPUs: $VCPUS"
echo ""

# Create disk image
DISK_PATH="/var/lib/libvirt/images/${VM_NAME}.qcow2"
echo -e "${GREEN}Creating disk image...${NC}"
sudo qemu-img create -f qcow2 "$DISK_PATH" "$DISK_SIZE"

# Create VM
echo -e "${GREEN}Creating VM...${NC}"
sudo virt-install \
    --name "$VM_NAME" \
    --ram "$MEMORY" \
    --vcpus "$VCPUS" \
    --disk path="$DISK_PATH",format=qcow2 \
    --cdrom "$ISO_PATH" \
    --os-variant linux2022 \
    --network default \
    --graphics spice \
    --noautoconsole

echo -e "${GREEN}Waiting for VM to be defined...${NC}"
sleep 2

# Stop VM if running (virt-install starts it)
echo -e "${GREEN}Stopping VM to add virtio-serial channel...${NC}"
sudo virsh destroy "$VM_NAME" 2>/dev/null || true
sleep 2

# Add virtio-serial channel
echo -e "${GREEN}Adding virtio-serial channel...${NC}"
CHANNEL_XML=$(cat <<EOF
<channel type='unix'>
  <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0'/>
  <target type='virtio' name='org.kvmmanager.agent.0'/>
</channel>
EOF
)

echo "$CHANNEL_XML" | sudo virsh attach-device "$VM_NAME" /dev/stdin --config

# Verify channel was added
echo -e "${GREEN}Verifying virtio-serial channel...${NC}"
sudo virsh dumpxml "$VM_NAME" | grep -A 3 "org.kvmmanager.agent.0" && \
    echo -e "${GREEN}✓ Virtio-serial channel configured${NC}" || \
    echo -e "${RED}✗ Failed to configure virtio-serial channel${NC}"

echo ""
echo -e "${GREEN}=== VM Created Successfully ===${NC}"
echo ""
echo "Next steps:"
echo "  1. Start the VM and install Linux:"
echo "     ${YELLOW}sudo virsh start $VM_NAME${NC}"
echo "     ${YELLOW}virt-viewer $VM_NAME${NC}"
echo ""
echo "  2. After installing Linux, copy the agent to the VM:"
echo "     ${YELLOW}scp target/release/kvmmanager-agent user@vm-ip:~/${NC}"
echo ""
echo "  3. Install the agent (see guest-agent/TESTING.md for details)"
echo ""
echo "  4. Test the agent:"
echo "     ${YELLOW}sudo socat - UNIX-CONNECT:/var/lib/libvirt/qemu/channel/target/${VM_NAME}.org.kvmmanager.agent.0${NC}"
echo "     Type: ${YELLOW}{\"jsonrpc\":\"2.0\",\"method\":\"ping\",\"id\":1}${NC}"
echo ""
