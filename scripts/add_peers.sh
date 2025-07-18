#!/usr/bin/env bash

# Script to manually add peers (their enodes) to each node

sleep 5

RETH0_IP=$(docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' reth0)
RETH1_IP=$(docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' reth1)
RETH2_IP=$(docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' reth2)

RETH0_ENODE=$(cast rpc --rpc-url 127.0.0.1:8545 admin_nodeInfo | jq -r .enode | sed "s/127\.0\.0\.1/${RETH0_IP}/")
RETH1_ENODE=$(cast rpc --rpc-url 127.0.0.1:18545 admin_nodeInfo | jq -r .enode | sed "s/127\.0\.0\.1/${RETH1_IP}/" )
RETH2_ENODE=$(cast rpc --rpc-url 127.0.0.1:28545 admin_nodeInfo | jq -r .enode | sed "s/127\.0\.0\.1/${RETH2_IP}/" )

echo "RETH0_ENODE: ${RETH0_ENODE}"
cast rpc --rpc-url 127.0.0.1:8545 admin_addTrustedPeer "${RETH1_ENODE}"
cast rpc --rpc-url 127.0.0.1:8545 admin_addTrustedPeer "${RETH2_ENODE}"
cast rpc --rpc-url 127.0.0.1:8545 admin_addPeer "${RETH1_ENODE}"
cast rpc --rpc-url 127.0.0.1:8545 admin_addPeer "${RETH2_ENODE}"

echo "RETH1_ENODE: ${RETH1_ENODE}"
cast rpc --rpc-url 127.0.0.1:18545 admin_addTrustedPeer "${RETH0_ENODE}"
cast rpc --rpc-url 127.0.0.1:18545 admin_addTrustedPeer "${RETH2_ENODE}"
cast rpc --rpc-url 127.0.0.1:18545 admin_addPeer "${RETH0_ENODE}"
cast rpc --rpc-url 127.0.0.1:18545 admin_addPeer "${RETH2_ENODE}"

echo "RETH2_ENODE: ${RETH2_ENODE}"  
cast rpc --rpc-url 127.0.0.1:28545 admin_addTrustedPeer "${RETH0_ENODE}"
cast rpc --rpc-url 127.0.0.1:28545 admin_addTrustedPeer "${RETH1_ENODE}"
cast rpc --rpc-url 127.0.0.1:28545 admin_addPeer "${RETH0_ENODE}"
cast rpc --rpc-url 127.0.0.1:28545 admin_addPeer "${RETH1_ENODE}"
