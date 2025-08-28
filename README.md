# Data Aggregation Service

A Rust-based microservice that processes LibraData from Firestore and generates time-based and categorical aggregations. Designed to run as a scheduled job on Google Cloud Run.

## Overview

This service:
- Fetches new LibraData entries from Firestore since the last processing run
- Generates aggregations by category, action, hour, and date
- Stores results in organized Firestore collections
- Tracks processing state to enable incremental updates

## Architecture

### Input
- **Source Collection**: `libra` - Contains LibraData entries from production systems
- **Metadata Collection**: `aggregates` - Contains `last_processed` timestamp tracking

### Output Collections
- `aggregates_categories` - Ingredient category counts
- `aggregates_actions` - Action type counts (Served, Refilled, etc.)
- `aggregates_time_hours` - Hourly aggregation counts
- `aggregates_time_dates` - Daily aggregation counts

### Processing Logic
- **Incremental**: Only processes entries newer than `last_processed` timestamp
- **Initial Run**: Processes all entries if no `last_processed` document exists
- **Idempotent**: Safe to run multiple times - uses upsert operations

## Local Development

### Prerequisites
- Rust 1.70+
- Firebase CLI
- Google Cloud SDK

### Setup
1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```

3. Start Firebase emulator:
   ```bash
   firebase emulators:start --only firestore --project back-of-house-backend
   ```

4. Create `.env` file:
   ```bash
   FIRESTORE_EMULATOR_HOST=127.0.0.1:8080
   ```

5. Run tests:
   ```bash
   cargo test
   ```

### Testing
The integration test (`tests/integration.rs`) demonstrates the full pipeline:
- Seeds test LibraData
- Processes aggregations
- Verifies output collections

## Production Deployment

### Prerequisites
- Google Cloud Project with billing enabled
- Firestore database configured
- Service account with appropriate permissions

### 1. Enable Required APIs
```bash
gcloud services enable run.googleapis.com
gcloud services enable scheduler.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable secretmanager.googleapis.com
```

### 2. Create Service Account
```bash
# Create service account
gcloud iam service-accounts create data-aggregation-sa \
  --display-name="Data Aggregation Service Account"

# Grant Firestore permissions
gcloud projects add-iam-policy-binding PROJECT_ID \
  --member="serviceAccount:data-aggregation-sa@PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/datastore.user"

# Grant Secret Manager access (if using secrets)
gcloud projects add-iam-policy-binding PROJECT_ID \
  --member="serviceAccount:data-aggregation-sa@PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

### 3. Create Dockerfile
Create `Dockerfile` in project root:
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/data-aggregation /usr/local/bin/data-aggregation

# Expose port (Cloud Run requires this)
EXPOSE 8080

CMD ["data-aggregation"]
```

### 4. Deploy to Cloud Run
```bash
# Build and deploy in one step
gcloud run deploy data-aggregation \
  --source . \
  --platform managed \
  --region us-west1 \
  --allow-unauthenticated \
  --service-account=data-aggregation-sa@back-of-house-backend.iam.gserviceaccount.com \
  --set-env-vars="PROJECT_ID=back-of-house-backend" \
  --memory=512Mi \
  --cpu=1 \
  --timeout=900s \
  --concurrency=1 \
  --max-instances=1
```

#### Environment Variables
- Remove `FIRESTORE_EMULATOR_HOST` for production
- Add any required configuration via `--set-env-vars`
- Use Secret Manager for sensitive values:
  ```bash
  --set-secrets="API_KEY=projects/back-of-house-backend/secrets/api-key:latest"
  ```

### 5. Set up Scheduled Execution

#### Create Cloud Scheduler Job
```bash
# Get the Cloud Run service URL
SERVICE_URL=$(gcloud run services describe data-aggregation \
  --region=us-west1 \
  --format="value(status.url)")

# Create cron job (runs every 6 hours)
gcloud scheduler jobs create http data-aggregation-cron \
  --schedule="0 */6 * * *" \
  --uri="$SERVICE_URL" \
  --http-method=POST \
  --oidc-service-account-email=data-aggregation-sa@back-of-house-backend.iam.gserviceaccount.com \
  --location=us-central1 \
  --description="Data aggregation processing job"
```

#### Schedule Options
- `"0 */6 * * *"` - Every 6 hours
- `"0 9 * * *"` - Daily at 9 AM UTC
- `"0 0 * * 0"` - Weekly on Sunday at midnight UTC
- `"0 2 * * *"` - Daily at 2 AM UTC (recommended for low-traffic hours)

### 6. Configure Secrets (if needed)

#### Create secrets in Secret Manager
```bash
# Example: API key for external service
gcloud secrets create api-key \
  --data-file=api-key.txt

# Grant access to service account
gcloud secrets add-iam-policy-binding api-key \
  --member="serviceAccount:data-aggregation-sa@PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

#### Update deployment with secrets
```bash
gcloud run services update data-aggregation \
  --region=us-central1 \
  --set-secrets="API_KEY=api-key:latest"
```

## Monitoring & Operations

### Logging
- View logs: `gcloud logs read "resource.type=cloud_run_revision AND resource.labels.service_name=data-aggregation"`
- Logs are automatically collected in Cloud Logging
- Set up log-based metrics for monitoring

### Monitoring
```bash
# Check service status
gcloud run services describe data-aggregation --region=us-central1

# View recent executions
gcloud scheduler jobs describe data-aggregation-cron --location=us-central1

# Manual trigger for testing
gcloud scheduler jobs run data-aggregation-cron --location=us-central1
```

### Alerts
Set up Cloud Monitoring alerts for:
- Service execution failures
- High memory usage
- Long execution times
- Firestore errors

## Troubleshooting

### Common Issues

#### Service won't start
- Check service account permissions
- Verify Firestore database exists
- Check container logs for Rust panics

#### No data processed
- Verify LibraData exists in `libra` collection
- Check timestamp formats match between test and production data
- Ensure action filters match your data (`Action::Served` vs `Action::Refilled`)

#### Permission errors
- Verify service account has `roles/datastore.user`
- Check IAM bindings: `gcloud projects get-iam-policy PROJECT_ID`

#### Cron job not triggering
- Verify Cloud Scheduler job exists and is enabled
- Check service URL is correct and accessible
- Verify OIDC service account email

### Useful Commands
```bash
# Check recent logs
gcloud logs tail "resource.type=cloud_run_revision AND resource.labels.service_name=data-aggregation"

# Manual service trigger
curl -X POST $SERVICE_URL \
  -H "Authorization: Bearer $(gcloud auth print-identity-token)"

# Check Firestore data
# (Use Firebase Console or gcloud firestore commands)

# Update service configuration
gcloud run services update data-aggregation \
  --region=us-central1 \
  --memory=1Gi
```

## Security Considerations

1. **Least Privilege**: Service account only has necessary Firestore permissions
2. **Network Security**: Consider VPC connector for additional isolation
3. **Secrets**: Use Secret Manager for sensitive configuration
4. **Audit Logging**: Enable Firestore audit logs for compliance
5. **Resource Limits**: Set appropriate CPU/memory limits

## Performance Tuning

### Cloud Run Configuration
- **Memory**: Start with 512Mi, increase if needed
- **CPU**: 1 CPU should be sufficient for most workloads
- **Timeout**: 15 minutes max (900s)
- **Concurrency**: 1 (ensures sequential processing)
- **Max Instances**: 1 (prevents concurrent runs)

### Optimization Tips
- Monitor execution time and memory usage
- Consider batching for large datasets
- Use Firestore composite indexes for complex queries
- Implement exponential backoff for retries

## Next Steps

1. **Monitoring Dashboard**: Create Cloud Monitoring dashboard for service metrics
2. **Alerting**: Set up alerts for failures and performance issues
3. **Backup Strategy**: Implement Firestore backup if not already configured
4. **Load Testing**: Test with production data volumes
5. **Multi-Region**: Consider deploying to multiple regions for availability
6. **Error Handling**: Add more sophisticated error handling and retry logic
7. **Metrics Export**: Consider exporting aggregation results to BigQuery for analytics

## Contributing

1. Make changes locally with tests
2. Test against Firebase emulator
3. Deploy to staging environment
4. Verify production deployment

## License

[Add your license information here]