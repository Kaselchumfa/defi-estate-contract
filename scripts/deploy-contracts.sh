#!/usr/bin/env bash
# Deploy one property's contracts to a Stellar network.
#
# Usage: scripts/deploy-contracts.sh <source-identity> <property-id> <name> <symbol> <max-shares> <payout-token-id> [network]
#
# Requires the Stellar CLI and a funded identity (stellar keys generate --global <name> --network testnet --fund).
# Records the deployed contract IDs in deployments/<network>.json keyed by property id, then copy them into the
# frontend and backend contract registries (src/shared/contracts/contracts.<network>.json).
set -euo pipefail

if [ "$#" -lt 6 ]; then
  sed -n '2,9p' "$0"
  exit 2
fi

SOURCE="$1" PROPERTY_ID="$2" NAME="$3" SYMBOL="$4" MAX_SHARES="$5" PAYOUT_TOKEN="$6" NETWORK="${7:-testnet}"
ADMIN="$(stellar keys address "$SOURCE")"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WASM_DIR="$SCRIPT_DIR/../target/wasm32v1-none/release"
DEPLOYMENTS_FILE="$SCRIPT_DIR/../deployments/$NETWORK.json"

"$SCRIPT_DIR/build-contracts.sh" >/dev/null

TOKEN_ID="$(stellar contract deploy --wasm "$WASM_DIR/property_token.wasm" --source "$SOURCE" --network "$NETWORK" -- \
  --admin "$ADMIN" --property_id "$PROPERTY_ID" --name "$NAME" --symbol "$SYMBOL" --max_supply "$MAX_SHARES")"

DISTRIBUTOR_ID="$(stellar contract deploy --wasm "$WASM_DIR/yield_distributor.wasm" --source "$SOURCE" --network "$NETWORK" -- \
  --admin "$ADMIN" --payout_token "$PAYOUT_TOKEN" --share_token "$TOKEN_ID")"

echo "propertyToken:    $TOKEN_ID"
echo "yieldDistributor: $DISTRIBUTOR_ID"

# Record the deployed IDs in deployments/<network>.json, keyed by property id.
mkdir -p "$(dirname "$DEPLOYMENTS_FILE")"
if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo '{}' >"$DEPLOYMENTS_FILE"
fi

TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE"' EXIT

jq \
  --arg propertyId "$PROPERTY_ID" \
  --arg tokenId "$TOKEN_ID" \
  --arg distributorId "$DISTRIBUTOR_ID" \
  '.contracts = (.contracts // {})
   | .contracts[$propertyId] = {
       propertyToken: $tokenId,
       yieldDistributor: $distributorId
     }' \
  "$DEPLOYMENTS_FILE" >"$TMP_FILE"

mv "$TMP_FILE" "$DEPLOYMENTS_FILE"
trap - EXIT

echo "Recorded contract IDs in deployments/$NETWORK.json under contracts.$PROPERTY_ID"