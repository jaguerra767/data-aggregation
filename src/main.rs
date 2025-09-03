use data_aggregation::error::Error;
use data_aggregation::firestore::client::process_aggregations;
use dotenv::dotenv;
use firestore::*;
use std::env;
use warp::Filter;

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

    
    
    println!("Starting data aggregation service for project: {}", project);

    // Create the aggregation route
    let aggregation_route = warp::path("aggregate").and(warp::post()).and_then(move || {
        let project = project.clone();
        async move {
            match run_aggregation(project).await {
                Ok(_) => {
                    println!("Data aggregation completed successfully");
                    Ok(warp::reply::with_status(
                        "Success",
                        warp::http::StatusCode::OK,
                    ))
                }
                Err(e) => {
                    eprintln!("Data aggregation failed: {}", e);
                    Err(warp::reject::reject())
                }
            }
        }
    });

    // Health check route
    let health_route = warp::path("health").and(warp::get()).map(|| "OK");

    // Root route for basic requests
    let root_route = warp::path::end()
        .and(warp::any())
        .map(|| "Data Aggregation Service is running");

    let routes = aggregation_route.or(health_route).or(root_route);

    println!("Server starting on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

async fn run_aggregation(project: String) -> Result<(), Error> {
    let db = FirestoreDb::new(project).await?;
    process_aggregations(&db).await?;
    Ok(())
}
