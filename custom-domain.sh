#!/bin/bash
set -euo pipefail

export HOSTED_ZONE_NAME="spritehub.xyz"
export HOSTED_ZONE_ID="Z0737778XRXCQ46UJWNQ"

DOMAIN="oidc.${HOSTED_ZONE_NAME}"

# Fly.io allocated IPs
A_IP="66.241.124.251"
AAAA_IP="2a09:8280:1::13a:7802:0"

echo "Configuring Route53 DNS records for ${DOMAIN} ..."

# Create/update A record (IPv4)
aws route53 change-resource-record-sets \
  --hosted-zone-id "${HOSTED_ZONE_ID}" \
  --change-batch "{
    \"Changes\": [
      {
        \"Action\": \"UPSERT\",
        \"ResourceRecordSet\": {
          \"Name\": \"${DOMAIN}.\",
          \"Type\": \"A\",
          \"TTL\": 300,
          \"ResourceRecords\": [
            {\"Value\": \"${A_IP}\"}
          ]
        }
      },
      {
        \"Action\": \"UPSERT\",
        \"ResourceRecordSet\": {
          \"Name\": \"${DOMAIN}.\",
          \"Type\": \"AAAA\",
          \"TTL\": 300,
          \"ResourceRecords\": [
            {\"Value\": \"${AAAA_IP}\"}
          ]
        }
      }
    ]
  }"

echo ""
echo "Done. Records now propagating:"
echo "  A    ${DOMAIN} → ${A_IP}"
echo "  AAAA ${DOMAIN} → ${AAAA_IP}"
echo ""
echo "Verify propagation with:"
echo "  dig +short A    ${DOMAIN}"
echo "  dig +short AAAA ${DOMAIN}"
echo ""
echo "Check Fly.io certificate validation with:"
echo "  fly certs check ${DOMAIN}"