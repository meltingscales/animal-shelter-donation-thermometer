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
