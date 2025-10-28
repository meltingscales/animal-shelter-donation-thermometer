# Animal Shelter Donation Thermometer

https://animal-shelter-donation-thermometer-163012697625.us-central1.run.app/faq

A simple, embeddable donation progress thermometer for animal shelter fundraising campaigns with a user-friendly web interface. Track team donations in real-time with a visual thermometer image.

## Features

- **Visual Thermometer Image** - Generate a thermometer PNG showing donation progress
- **Web Interface** - Home page, FAQ, and Admin Portal with responsive design
- **Multiple Teams** - Track donations from multiple fundraising teams
- **Organization Customization** - Set your organization name and campaign title
- **Persistent Storage** - Data persists using Google Cloud Firestore
- **OpenAPI Documentation** - Full REST API with Swagger UI
- **Admin Portal** - Web-based interface for managing configuration and uploading CSV data
- **Privacy-Focused** - No tracking, no analytics, just simple fundraising tools
- **GCP Cloud Run ready** - Optimized for serverless deployment

## API Endpoints

### Public Endpoints

- `GET /` - Home page with thermometer display and team leaderboard
- `GET /faq` - Frequently asked questions page
- `GET /admin` - Admin portal (web interface)
- `GET /thermometer.png` - Donation thermometer image (PNG, embeddable)
- `GET /config` - Current thermometer configuration (JSON)
- `GET /health` - Health check endpoint
- `GET /openapi` - Swagger UI API documentation

### Admin Endpoints

Require `Authorization` header with `THERMOMETER_EDIT_KEY`

- `POST /admin/upload` - Upload CSV with team donation data
- `POST /admin/config` - Update configuration (JSON - includes organization name, title, goal, teams)

## Setup

### Prerequisites

- Rust 1.83 or later
- Docker (for containerization)
- Google Cloud SDK (for GCP deployment)
- `just` command runner (optional but recommended)

### Environment Variables

- `GCP_PROJECT` - Google Cloud Project ID (enables Firestore storage for persistence)
- `THERMOMETER_EDIT_KEY` - UUID for authenticating admin requests (auto-generated if not set)
- `BASE_URL` - Base URL for the service (default: `http://localhost:8080`)
- `PORT` - Server port (default: 8080)

### Local Development

1. Clone the repository:
```bash
cd /home/melty/Git/animal-shelter-domation-thermometer
```

2. Set your edit key:
```bash
export THERMOMETER_EDIT_KEY=$(uuidgen)
echo "Your edit key: $THERMOMETER_EDIT_KEY"
```

3. Run the server:
```bash
just run
# or
cargo run
```

The server will start on http://localhost:8080

4. Open the web interface:
- Home page: http://localhost:8080/
- Admin portal: http://localhost:8080/admin
- API docs: http://localhost:8080/openapi

5. Test the API:
```bash
# Get config
curl http://localhost:8080/config

# Upload CSV (replace YOUR_KEY with your edit key)
curl -X POST http://localhost:8080/admin/upload \
  -H "Authorization: Bearer YOUR_KEY" \
  -F "file=@teams.csv"
```

### CSV Format

The CSV should have the following columns:

```csv
name,image_url,total_raised
Team Alpha,https://example.com/alpha.jpg,1250.50
Team Beta,https://example.com/beta.jpg,2340.00
Team Gamma,,890.75
```

- `name` - Team name (required)
- `image_url` - URL to team logo/image (optional)
- `total_raised` - Amount raised in dollars (required)

### Configuration JSON Format

```json
{
  "organization_name": "Community Animal Rescue Effort Skokie",
  "title": "Annual Fundraising Drive 2025",
  "goal": 50000.0,
  "teams": [
    {
      "name": "Team Alpha",
      "image_url": "https://example.com/alpha.jpg",
      "total_raised": 1250.50
    }
  ],
  "last_updated": "2025-10-27T00:00:00Z"
}
```

You can also update the organization name, title, and goal through the web-based Admin Portal at `/admin`.

## Deployment

### Local with Firestore (Recommended for Testing)

```bash
# Set your GCP project ID
export GCP_PROJECT="your-gcp-project-id"

# Setup Firestore (one-time setup)
just firestore-setup

# Run locally with Firestore
just run
```

### Docker

Build and run locally:
```bash
just docker-build
just docker-run
```

### Google Cloud Run (Production)

#### Complete Setup (First Time)

```bash
# Set your GCP project ID
export GCP_PROJECT="your-project-id"
export GCP_REGION="us-central1"  # optional, defaults to us-central1

# Build, setup Firestore, and deploy in one command
just gcp-setup-all
```

This will:
1. Build and push the Docker image to Google Container Registry
2. Setup Firestore database in your GCP project
3. Deploy to Cloud Run with Firestore environment variable

#### Update Existing Deployment

```bash
# Build and deploy
just gcp-push
just gcp-deploy-firestore
```

#### Optional: Set Custom Edit Key

```bash
gcloud run services update animal-shelter-donation-thermometer \
  --update-env-vars THERMOMETER_EDIT_KEY=your-uuid-here \
  --region us-central1 \
  --project $GCP_PROJECT
```

#### Get Service URL

```bash
just gcp-url
```

## Usage in Emails

To embed the thermometer in an email:

```html
<img src="https://your-service-url.run.app/thermometer.png" alt="Donation Progress">
```

The image uses cache-busting headers to ensure emails always show the latest version.

## Storage

### Firestore (Production - Recommended)

When `GCP_PROJECT` is set, the service uses Google Cloud Firestore for persistent storage:
- Data persists across server restarts
- Automatic scaling
- Generous free tier (50K reads/day, 20K writes/day)
- No configuration needed beyond project ID

**Storage Details:**
- Collection: `thermometer_configs`
- Document ID: `current_config`

### In-Memory (Development)

When `GCP_PROJECT` is not set, data is stored in memory:
- Fast and simple for development
- Data is lost when server restarts
- No external dependencies

## Justfile Commands

### Development
- `just help` - List all available commands
- `just build` - Build the project
- `just run` - Run the server locally
- `just test` - Run tests
- `just fmt` - Format code

### Docker
- `just docker-build` - Build Docker image
- `just docker-run` - Run Docker container locally
- `just docker-stop` - Stop running containers

### GCP Deployment
- `just gcp-push` - Build and push to Google Container Registry
- `just gcp-deploy-firestore` - Deploy to Cloud Run with Firestore
- `just gcp-setup-all` - Complete setup (build + Firestore + deploy)
- `just gcp-logs` - View Cloud Run logs
- `just gcp-url` - Get service URL

### Firestore Management
- `just firestore-setup` - Setup Firestore in GCP project
- `just firestore-status` - Check Firestore status
- `just firestore-view` - View Firestore console URL
- `just firestore-clear` - Delete all Firestore data (⚠️ dangerous)

### Security & Key Management
- `just generate-key` - Generate a new THERMOMETER_EDIT_KEY (UUID)
- `just set-cloud-key <key>` - Set a specific key in Cloud Run
- `just rotate-cloud-key` - Generate and set a new key in Cloud Run (with confirmation)
- `just show-cloud-env` - Show current environment variables in Cloud Run
- `just remove-cloud-key` - Remove custom key (use auto-generated key)

## Architecture

### Technology Stack

- **Backend:** Rust with Axum web framework
- **Storage:** Google Cloud Firestore (NoSQL, optional)
- **Templates:** Askama (compile-time HTML templates)
- **API Documentation:** OpenAPI 3.0 + Swagger UI
- **Styling:** Responsive CSS (mobile-friendly)
- **Deployment:** Docker + Google Cloud Run

### Project Structure

```
.
├── src/
│   ├── main.rs          # Main application and routes
│   └── storage.rs       # Firestore/in-memory storage
├── templates/           # HTML templates
│   ├── base.html        # Base template with navbar
│   ├── home.html        # Home page with thermometer
│   ├── faq.html         # FAQ page
│   └── admin.html       # Admin portal
├── static/
│   └── styles.css       # Responsive CSS styles
├── Cargo.toml           # Rust dependencies
├── Dockerfile           # Container configuration
└── justfile             # Build and deployment commands
```

## FAQ

### How do I get my authorization key?

**For local development:**
```bash
# Generate a new key
just generate-key

# Or let the server auto-generate one (check logs)
just run
```

**For Cloud Run:**
```bash
# Generate and set a new key
just generate-key
just set-cloud-key <your-key>

# Or rotate the key (generates and sets in one command)
just rotate-cloud-key
```

The key will be displayed in the server logs on startup if not set via `THERMOMETER_EDIT_KEY`:
```
WARN: THERMOMETER_EDIT_KEY not set, generated new key: <your-key-here>
```

**Save this key securely** - you'll need it for admin operations.

### Can I customize the organization name and campaign title?

Yes! Visit the Admin Portal at `/admin` and use the configuration form to set:
- Organization name (e.g., "Community Animal Rescue Effort Skokie")
- Campaign title (e.g., "Annual Fundraising Drive 2025")
- Fundraising goal

### How much does Firestore cost?

Firestore has a generous free tier:
- 50,000 document reads/day
- 20,000 document writes/day
- 1 GB storage

For a typical donation thermometer, this should cover most use cases for free.

### Can I run this without Google Cloud?

Yes! The service works perfectly with in-memory storage for local/small deployments. Simply don't set the `GCP_PROJECT` environment variable.

### How do I implement actual thermometer image generation?

The thermometer PNG endpoint currently returns a placeholder. To implement actual image generation, you can:
1. Use the `image` crate for PNG generation
2. Use `resvg` for SVG to PNG conversion
3. Design the thermometer based on the example image (see `thermometer_example_2024.jpg`)

## TODO

- [ ] Implement actual PNG generation based on config (currently returns placeholder)
- [ ] Add image rendering library (e.g., `image`, `resvg` for SVG->PNG)
- [ ] Design thermometer visualization matching the example image
- [ ] Add metrics/monitoring
- [ ] Add rate limiting for admin endpoints

## License

MIT
