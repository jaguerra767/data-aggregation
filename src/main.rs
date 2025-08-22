use data_aggregation::error::Error;
use data_aggregation::firestore::client::process_aggregations;
use dotenv::dotenv;
use firestore::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load .env for local development only
    dotenv().ok();
    
    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    
    // Get project ID from environment variable
    let project = env::var("GOOGLE_CLOUD_PROJECT")?;
    
    println!("Starting data aggregation for project: {}", project);
    
    let db = FirestoreDb::new(project).await?;
    
    // Run the aggregation process
    process_aggregations(&db).await?;
    
    println!("Data aggregation completed successfully");
    Ok(())
}
