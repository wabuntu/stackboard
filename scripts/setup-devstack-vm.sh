#!/usr/bin/env bash
# One-shot: launches an isolated multipass VM and installs a minimal
# DevStack (Keystone + Nova + Neutron + Glance + Cinder) inside it, for
# testing stackboard against a real OpenStack API.
#
# Runs DevStack inside a disposable VM rather than on this host directly —
# DevStack makes invasive, hard-to-undo changes (system packages, MySQL,
# RabbitMQ, network config) to whatever it's installed on, and its own
# docs recommend a throwaway machine. `multipass delete --purge devstack`
# tears the whole thing down with nothing left behind on the host.
set -euo pipefail

VM_NAME=devstack
VM_CPUS=8
VM_MEM=12G
VM_DISK=60G

if ! command -v multipass >/dev/null; then
    echo "multipass not found. Install it first:"
    echo "  sudo snap install multipass"
    exit 1
fi

if multipass info "$VM_NAME" >/dev/null 2>&1; then
    echo "A VM named '$VM_NAME' already exists. Delete it first if you want a fresh install:"
    echo "  multipass delete --purge $VM_NAME"
    exit 1
fi

echo "==> Launching $VM_NAME (${VM_CPUS} CPUs, ${VM_MEM} RAM, ${VM_DISK} disk, Ubuntu 24.04)..."
multipass launch 24.04 --name "$VM_NAME" --cpus "$VM_CPUS" --memory "$VM_MEM" --disk "$VM_DISK"

VM_IP=$(multipass info "$VM_NAME" --format json | python3 -c "import json,sys; print(json.load(sys.stdin)['info']['$VM_NAME']['ipv4'][0])")
echo "==> VM IP: $VM_IP"

ADMIN_PASSWORD=$(openssl rand -hex 12)
SERVICE_PASSWORD=$(openssl rand -hex 12)
DATABASE_PASSWORD=$(openssl rand -hex 12)
RABBIT_PASSWORD=$(openssl rand -hex 12)

echo "==> Provisioning the stack user and DevStack inside the VM..."
multipass exec "$VM_NAME" -- sudo bash -c "
set -euo pipefail
useradd -s /bin/bash -d /opt/stack -m stack
chmod +x /opt/stack
echo 'stack ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/stack
apt-get update -qq
apt-get install -y -qq git
"

multipass exec "$VM_NAME" -- sudo -u stack bash -c "
set -euo pipefail
cd /opt/stack
git clone --depth 1 https://opendev.org/openstack/devstack
cd devstack
cat > local.conf <<EOF
[[local|localrc]]
ADMIN_PASSWORD=${ADMIN_PASSWORD}
DATABASE_PASSWORD=${DATABASE_PASSWORD}
RABBIT_PASSWORD=${RABBIT_PASSWORD}
SERVICE_PASSWORD=${SERVICE_PASSWORD}
HOST_IP=${VM_IP}
EOF
"

echo "==> Running stack.sh — this genuinely takes 20-40 minutes, grab a coffee."
multipass exec "$VM_NAME" -- sudo -u stack bash -c "cd /opt/stack/devstack && ./stack.sh"

CREDS_FILE="$HOME/.devstack-${VM_NAME}-credentials"
cat > "$CREDS_FILE" <<EOF
# DevStack credentials for VM '$VM_NAME' — generated $(date -Iseconds)
export OS_AUTH_URL=http://${VM_IP}/identity
export OS_USERNAME=admin
export OS_PASSWORD=${ADMIN_PASSWORD}
export OS_PROJECT_NAME=admin
export OS_USER_DOMAIN_NAME=Default
export OS_PROJECT_DOMAIN_NAME=Default
export OS_REGION_NAME=RegionOne
EOF
chmod 600 "$CREDS_FILE"

echo
echo "==> Done."
echo "Horizon dashboard: http://${VM_IP}/ (login: admin / ${ADMIN_PASSWORD})"
echo "Credentials for stackboard saved to: $CREDS_FILE"
echo
echo "To point stackboard at this cloud:"
echo "  source $CREDS_FILE"
echo "  stackboard"
echo
echo "To tear it all down later, with nothing left on this host:"
echo "  multipass delete --purge $VM_NAME"
