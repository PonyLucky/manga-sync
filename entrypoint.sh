#!/bin/sh
CERT_DIR="/usr/local/bin/secret/ssl"

# Create SSL directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Generate self-signed TLS certificate if it doesn't exist
if [ ! -f "$CERT_DIR/cert.pem" ]; then
    echo "Generating self-signed TLS certificate..."
    openssl req -x509 -newkey rsa:4096 -keyout "$CERT_DIR/key.pem" \
        -out "$CERT_DIR/cert.pem" -days 365 -nodes \
        -subj "/CN=manga-sync/O=Local/C=US"
    echo "TLS certificate generated successfully."
fi

exec ./manga-sync
