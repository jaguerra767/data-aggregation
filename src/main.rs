use data_aggregation::error::Error;
use data_aggregation::firestore::client::{process_aggregations, read_locations, LocationData};
use data_aggregation::query::LocationQuery;
use dotenv::dotenv;
use firestore::*;
use std::env;
use warp::{Filter, Rejection, Reply};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load .env for local development only
    dotenv().ok();

    // Debug: Check if the emulator host env var is set correctly
    println!(
        "FIRESTORE_EMULATOR_HOST: {:?}",
        std::env::var("FIRESTORE_EMULATOR_HOST")
    );

    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Get project ID from environment variable
    let project = env::var("GOOGLE_CLOUD_PROJECT")?;

    println!("Starting data aggregation service for project: {}", project);

    let db = FirestoreDb::new(&project).await?;

    let with_db = warp::any().map(move || db.clone());

    // Create the aggregation route
    let aggregation_route = warp::path("aggregate")
        .and(warp::post())
        .and(with_db.clone())
        .and_then(run_aggregation_handler);

    // Create the locations route
    let locations_route = warp::path("locations")
        .and(warp::query::<LocationQuery>())
        .and(warp::get())
        .and(with_db.clone())
        .and_then(handle_location_query);

    // Health check route
    let health_route = warp::path("health").and(warp::get()).map(|| "OK");

    // Root route for basic requests
    let root_route = warp::path::end()
        .and(warp::any())
        .map(|| "Data Aggregation Service is running");

    let routes = aggregation_route
        .or(health_route)
        .or(root_route)
        .or(locations_route);

    println!("Server starting on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

async fn run_aggregation_handler(db: FirestoreDb) -> Result<impl Reply, Rejection> {
    match process_aggregations(&db).await {
        Ok(_) => {
            println!("Data aggregation completed successfully");
            Ok(warp::reply::with_status(
                "Success",
                warp::http::StatusCode::OK,
            ))
        }
        Err(e) => {
            eprintln!("Data aggregation failed: {:?}", e);
            Err(warp::reject::custom(e))
        }
    }
}

async fn handle_location_query(
    param: LocationQuery,
    db: FirestoreDb,
) -> Result<impl Reply, Rejection> {
    match read_locations(&db).await {
        Ok(location_data) => {
            let matching_locations = location_data
                .iter()
                .filter(|location_data| {
                    param.serial_number.as_ref().is_none_or(|serial_number| {
                        *serial_number == location_data.device.serial_number
                    }) && param
                        .location
                        .as_ref()
                        .is_none_or(|location| location == &location_data.location)
                })
                .collect::<Vec<&LocationData>>();
            let reply = warp::reply::json(&matching_locations);
            Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
        }
        Err(e) => {
            eprintln!("Location query failed: {:?}", e);
            Err(warp::reject::custom(e))
        }
    }
}
