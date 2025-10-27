# Animal Shelter Donation Thermometer

A simple web API that serves a static PNG file displaying donation campaign progress for an animal shelter. Designed to be embedded in emails.

## Features

- **Static PNG endpoint** (`/thermometer.png`) - Embeddable image showing donation progress
- **Admin API** - Secure endpoint to upload donation data via CSV
- **Configuration API** - Update campaign title, goal, team info, etc.
- **Public config endpoint** - Get current campaign stats as JSON
- **GCP Cloud Run ready** - Optimized Dockerfile and deployment configuration

## API Endpoints

### Public Endpoints

- `GET /` - API documentation
- `GET /thermometer.png` - Donation thermometer image (PNG)
- `GET /config` - Current thermometer configuration (JSON)
- `GET /health` - Health check

### Admin Endpoints

Require `Authorization` header with `THERMOMETER_EDIT_KEY`

- `POST /admin/upload` - Upload CSV with team donation data
- `POST /admin/config` - Update configuration (JSON)

## Setup

### Prerequisites

- Rust 1.83 or later
- Docker (for containerization)
- Google Cloud SDK (for GCP deployment)
- `just` command runner (optional but recommended)

### Environment Variables

- `THERMOMETER_EDIT_KEY` - UUID for authenticating admin requests (required for production)
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

4. Test the endpoints:
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
  "title": "2025 Annual Fundraiser",
  "goal": 50000.0,
  "teams": [
    {
      "name": "Team Alpha",
      "image_url": "https://example.com/alpha.jpg",
      "total_raised": 1250.50
    }
  ]
}
```

## Deployment

### Docker

Build and run locally:
```bash
just docker-build
just docker-run
```

### Google Cloud Run

1. Set your GCP project ID:
```bash
export GCP_PROJECT="your-project-id"
```

2. Build, push, and deploy:
```bash
just gcp-deploy-all
```

3. Set the edit key as an environment variable in Cloud Run:
```bash
gcloud run services update animal-shelter-donation-thermometer \
  --update-env-vars THERMOMETER_EDIT_KEY=your-uuid-here \
  --region us-central1 \
  --project $GCP_PROJECT
```

4. Get the service URL:
```bash
just gcp-url
```

## Usage in Emails

To embed the thermometer in an email:

```html
<img src="https://your-service-url.run.app/thermometer.png" alt="Donation Progress">
```

The image uses cache-busting headers to ensure emails always show the latest version.

## TODO

- [ ] Implement actual PNG generation based on config (currently returns placeholder)
- [ ] Add image rendering library (e.g., `image`, `resvg` for SVG->PNG)
- [ ] Design thermometer visualization
- [ ] Add persistence layer (e.g., Cloud Firestore, PostgreSQL)
- [ ] Add metrics/monitoring
- [ ] Add rate limiting for admin endpoints

## License

MIT
