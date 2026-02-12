# List available recipes
help:
    @just --list

# Build the project in release mode
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Run the web server locally
run:
    cargo run

# Run the web server in release mode (faster)
run-release:
    cargo run --release

# Format Rust code
fmt:
    cargo fmt

# Clean up generated files and build artifacts
clean:
    cargo clean

# Docker operations
# ================

# Build Docker image
docker-build:
    docker build -t animal-shelter-donation-thermometer:latest .

# Build Docker image with a specific tag
docker-build-tag tag:
    docker build -t animal-shelter-donation-thermometer:{{tag}} .

# Run Docker container locally
docker-run port="8080":
    docker run -p {{port}}:8080 animal-shelter-donation-thermometer:latest

# Stop all running containers for this project
docker-stop:
    docker ps -q --filter ancestor=animal-shelter-donation-thermometer:latest | xargs -r docker stop

# Remove Docker image
docker-clean:
    docker rmi animal-shelter-donation-thermometer:latest

# GCP Deployment
# ==============

# Set these variables for your GCP project
GCP_PROJECT := env_var_or_default("GCP_PROJECT", "your-gcp-project-id")
GCP_REGION := env_var_or_default("GCP_REGION", "us-central1")
SERVICE_NAME := "animal-shelter-donation-thermometer"

# Build and push Docker image to Google Container Registry
gcp-push:
    docker build -t gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:latest .
    docker push gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:latest

# Build and push Docker image with a specific tag
gcp-push-tag tag:
    docker build -t gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:{{tag}} .
    docker push gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:{{tag}}

# Deploy to Google Cloud Run
gcp-deploy:
    gcloud run deploy {{SERVICE_NAME}} \
        --image gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:latest \
        --platform managed \
        --region {{GCP_REGION}} \
        --allow-unauthenticated \
        --port 8080 \
        --project {{GCP_PROJECT}}

# Deploy a specific tagged version to Cloud Run
gcp-deploy-tag tag:
    gcloud run deploy {{SERVICE_NAME}} \
        --image gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:{{tag}} \
        --platform managed \
        --region {{GCP_REGION}} \
        --allow-unauthenticated \
        --port 8080 \
        --project {{GCP_PROJECT}}

# Build, push, and deploy to GCP in one command
gcp-deploy-all:
    just gcp-push
    just gcp-deploy

# View Cloud Run service logs
gcp-logs:
    gcloud run services logs read {{SERVICE_NAME}} --region {{GCP_REGION}} --project {{GCP_PROJECT}}

# Get Cloud Run service URL
gcp-url:
    gcloud run services describe {{SERVICE_NAME}} --region {{GCP_REGION}} --project {{GCP_PROJECT}} --format 'value(status.url)'

# Firestore Setup
# ===============

# Setup Firestore in GCP project
firestore-setup:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Setting up Firestore for project: {{GCP_PROJECT}}"

    # Check if Firestore is already enabled
    if gcloud firestore databases list --project={{GCP_PROJECT}} 2>/dev/null | grep -q "name:"; then
        echo "✓ Firestore is already enabled for project {{GCP_PROJECT}}"
    else
        echo "Enabling Firestore..."
        # Create Firestore database in Native mode
        gcloud firestore databases create \
            --location={{GCP_REGION}} \
            --type=firestore-native \
            --project={{GCP_PROJECT}}
        echo "✓ Firestore database created"
    fi

    echo ""
    echo "Firestore is ready! Your service will automatically use it when GCP_PROJECT is set."
    echo "Collection name: thermometer_configs"
    echo "Document ID: current_config"

# Check Firestore status
firestore-status:
    @echo "Checking Firestore status for project: {{GCP_PROJECT}}"
    @gcloud firestore databases list --project={{GCP_PROJECT}} || echo "Firestore not enabled. Run 'just firestore-setup' to enable it."

# View Firestore data (requires firestore emulator or gcloud alpha)
firestore-view:
    @echo "To view Firestore data, use the Firebase Console:"
    @echo "https://console.firebase.google.com/project/{{GCP_PROJECT}}/firestore"

# Delete all Firestore data (DANGEROUS - use with caution)
firestore-clear:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "⚠️  WARNING: This will delete all data in the thermometer_configs collection!"
    read -p "Are you sure? (type 'yes' to confirm): " confirm
    if [ "$confirm" = "yes" ]; then
        echo "Deleting Firestore data..."
        gcloud firestore documents delete \
            --collection thermometer_configs \
            --document current_config \
            --project={{GCP_PROJECT}} \
            --quiet || echo "Document may not exist yet"
        echo "✓ Firestore data cleared"
    else
        echo "Cancelled"
    fi

# Deploy to Cloud Run with Firestore enabled
gcp-deploy-firestore: firestore-setup
    @echo "Deploying to Cloud Run with Firestore enabled..."
    gcloud run deploy {{SERVICE_NAME}} \
        --image gcr.io/{{GCP_PROJECT}}/{{SERVICE_NAME}}:latest \
        --platform managed \
        --region {{GCP_REGION}} \
        --allow-unauthenticated \
        --port 8080 \
        --set-env-vars "GCP_PROJECT={{GCP_PROJECT}}" \
        --project {{GCP_PROJECT}}
    @echo "✓ Service deployed with Firestore integration"

# Complete setup: Build, push, setup Firestore, and deploy
gcp-setup-all: gcp-push firestore-setup gcp-deploy-firestore
    @echo ""
    @echo "✓ Complete setup finished!"
    @echo ""
    @echo "Service URL:"
    @just gcp-url

# Security & Key Management
# =========================

# Generate a new THERMOMETER_EDIT_KEY (UUID)
generate-key:
    #!/usr/bin/env bash
    set -euo pipefail

    # Try different UUID generation methods
    if command -v uuidgen &> /dev/null; then
        NEW_KEY=$(uuidgen | tr '[:upper:]' '[:lower:]')
    elif command -v python3 &> /dev/null; then
        NEW_KEY=$(python3 -c 'import uuid; print(uuid.uuid4())')
    else
        echo "Error: Cannot generate UUID. Please install 'uuidgen' or 'python3'"
        exit 1
    fi

    echo "Generated new THERMOMETER_EDIT_KEY:"
    echo ""
    echo "  $NEW_KEY"
    echo ""
    echo "To use this key locally:"
    echo "  export THERMOMETER_EDIT_KEY=\"$NEW_KEY\""
    echo ""
    echo "To set this key in Cloud Run:"
    echo "  just set-cloud-key $NEW_KEY"

# Set THERMOMETER_EDIT_KEY in Cloud Run
set-cloud-key key:
    @echo "Setting THERMOMETER_EDIT_KEY in Cloud Run..."
    gcloud run services update {{SERVICE_NAME}} \
        --update-env-vars "THERMOMETER_EDIT_KEY={{key}}" \
        --region {{GCP_REGION}} \
        --project {{GCP_PROJECT}}
    @echo "✓ Key updated successfully"
    @echo ""
    @echo "Save this key securely - you'll need it for admin operations:"
    @echo "  {{key}}"

# Generate and set a new key in Cloud Run (key rotation)
rotate-cloud-key:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🔄 Rotating THERMOMETER_EDIT_KEY in Cloud Run..."
    echo ""

    # Generate new key
    if command -v uuidgen &> /dev/null; then
        NEW_KEY=$(uuidgen | tr '[:upper:]' '[:lower:]')
    elif command -v python3 &> /dev/null; then
        NEW_KEY=$(python3 -c 'import uuid; print(uuid.uuid4())')
    else
        echo "Error: Cannot generate UUID. Please install 'uuidgen' or 'python3'"
        exit 1
    fi

    echo "Generated new key: $NEW_KEY"
    echo ""
    echo "⚠️  WARNING: This will invalidate the old key. Any existing admin sessions will need to use the new key."
    read -p "Continue? (type 'yes' to confirm): " confirm

    if [ "$confirm" = "yes" ]; then
        echo ""
        echo "Updating Cloud Run service..."
        gcloud run services update {{SERVICE_NAME}} \
            --update-env-vars "THERMOMETER_EDIT_KEY=$NEW_KEY" \
            --region {{GCP_REGION}} \
            --project {{GCP_PROJECT}}
        echo ""
        echo "✓ Key rotated successfully!"
        echo ""
        echo "🔑 NEW THERMOMETER_EDIT_KEY:"
        echo "  $NEW_KEY"
        echo ""
        echo "⚠️  Save this key securely - the old key is no longer valid!"
    else
        echo "Cancelled"
    fi

# Show current environment variables in Cloud Run (key is hidden by default)
show-cloud-env:
    @echo "Current environment variables for {{SERVICE_NAME}}:"
    @gcloud run services describe {{SERVICE_NAME}} \
        --region {{GCP_REGION}} \
        --project {{GCP_PROJECT}} \
        --format='value(spec.template.spec.containers[0].env)'

# Remove THERMOMETER_EDIT_KEY from Cloud Run (use auto-generated key)
remove-cloud-key:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "⚠️  WARNING: This will remove the custom key. The service will auto-generate a new one on next restart."
    echo "You'll need to check the Cloud Run logs to get the new auto-generated key."
    read -p "Continue? (type 'yes' to confirm): " confirm

    if [ "$confirm" = "yes" ]; then
        gcloud run services update {{SERVICE_NAME}} \
            --remove-env-vars THERMOMETER_EDIT_KEY \
            --region {{GCP_REGION}} \
            --project {{GCP_PROJECT}}
        echo ""
        echo "✓ Custom key removed"
        echo ""
        echo "The service will auto-generate a new key on next restart."
        echo "Check the logs with: just gcp-logs"
    else
        echo "Cancelled"
    fi

# Systemd Service Setup (for GCP VM)
# ====================================

# Install as systemd service running on port 3002
# Run with sudo
systemd-install:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ $EUID -ne 0 ]]; then
        echo "Error: This recipe must be run as root (use sudo)."
        exit 1
    fi

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    SERVICE_NAME="animal-shelter-thermometer"
    PORT="${PORT:-3002}"
    REPO_DIR="${REPO_DIR:-${SCRIPT_DIR}}"
    USER="${SUDO_USER:-root}"

    echo "Installing systemd service: ${SERVICE_NAME}"

    # Build release binary first
    echo "Building release binary..."
    (cd "${REPO_DIR}" && cargo build --release)

    # Copy and template service file
    sed -e "s|USER_PLACEHOLDER|${USER}|g" \
        -e "s|REPO_DIR_PLACEHOLDER|${REPO_DIR}|g" \
        "${SCRIPT_DIR}/systemd/${SERVICE_NAME}.service" \
        > /etc/systemd/system/${SERVICE_NAME}.service

    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable ${SERVICE_NAME}
    systemctl restart ${SERVICE_NAME}

    echo "Service installed and started on port ${PORT}!"
    echo ""
    echo "Commands:"
    echo "  sudo systemctl status ${SERVICE_NAME}"
    echo "  sudo systemctl restart ${SERVICE_NAME}"
    echo "  sudo journalctl -u ${SERVICE_NAME} -f"

# Uninstall systemd service
# Run with sudo
systemd-uninstall:
    #!/usr/bin/env bash
    SERVICE_NAME="animal-shelter-thermometer"

    if [[ $EUID -ne 0 ]]; then
        echo "Error: This recipe must be run as root (use sudo)."
        exit 1
    fi

    echo "Stopping and disabling ${SERVICE_NAME}..."
    systemctl stop ${SERVICE_NAME} 2>/dev/null || true
    systemctl disable ${SERVICE_NAME} 2>/dev/null || true
    rm -f /etc/systemd/system/${SERVICE_NAME}.service
    systemctl daemon-reload
    echo "Service uninstalled."

# Show service status
systemd-status:
    #!/usr/bin/env bash
    SERVICE_NAME="${SERVICE_NAME:-animal-shelter-thermometer}"
    systemctl status ${SERVICE_NAME}

# View service logs
systemd-logs:
    #!/usr/bin/env bash
    SERVICE_NAME="${SERVICE_NAME:-animal-shelter-thermometer}"
    journalctl -u ${SERVICE_NAME} -f

# Restart the service
systemd-restart:
    #!/usr/bin/env bash
    SERVICE_NAME="${SERVICE_NAME:-animal-shelter-thermometer}"
    systemctl restart ${SERVICE_NAME}
    systemctl status ${SERVICE_NAME}
