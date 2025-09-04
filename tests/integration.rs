use data_aggregation::error::Error;
use data_aggregation::firestore::client::process_aggregations;
use firestore::FirestoreDb;
use menu::action::Action;
use menu::device::{Device, Model};
use menu::libra_data::LibraData;
use time::OffsetDateTime;

async fn seed_libra_data(db: &FirestoreDb) -> Result<(), Error> {
    let data = vec![
        LibraData {
            device: Device {
                model: Model::LibraV0,
                serial_number: "Lib298190".to_string(),
            },
            location: "Caldo Office".to_string(),
            ingredient: "Plastic French Fries".to_string(),
            data_action: Action::Served,
            amount: 0.0,
            timestamp: OffsetDateTime::now_utc(),
        },
        LibraData {
            device: Device {
                model: Model::LibraV0,
                serial_number: "Lib298191".to_string(),
            },
            location: "Caldo Office".to_string(),
            ingredient: "Fake Broccoli".to_string(),
            data_action: Action::Served,
            amount: 0.0,
            timestamp: OffsetDateTime::now_utc(),
        },
        LibraData {
            device: Device {
                model: Model::LibraV0,
                serial_number: "Lib298192".to_string(),
            },
            location: "Lounge".to_string(),
            ingredient: "Kettle Chips".to_string(),
            data_action: Action::Served,
            amount: 0.0,
            timestamp: OffsetDateTime::now_utc(),
        },
        LibraData {
            device: Device {
                model: Model::LibraV0,
                serial_number: "Lib298193".to_string(),
            },
            location: "Lounge".to_string(),
            ingredient: "Popcorn".to_string(),
            data_action: Action::Served,
            amount: 0.0,
            timestamp: OffsetDateTime::now_utc(),
        },
    ];
    for d in data {
        println!("Inserting data: {:?}", d);
        // Convert to FirestoreLibraData for proper timestamp serialization
        let firestore_data = data_aggregation::firestore::client::FirestoreLibraData::from(d);
        let result = db
            .fluent()
            .insert()
            .into("libra")
            .generate_document_id()
            .object(&firestore_data)
            .execute::<()>()
            .await?;
        println!("Insert result: {:?}", result);
    }
    Ok(())
}

#[tokio::test]
async fn test_aggregation() -> Result<(), Error> {
    // Try to load .env and see if it succeeds
    dotenv::dotenv().ok();

    // Debug: Check if the env var is set correctly
    println!(
        "FIRESTORE_EMULATOR_HOST: {:?}",
        std::env::var("FIRESTORE_EMULATOR_HOST")
    );

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let db = FirestoreDb::new("back-of-house-backend".to_string()).await?;

    seed_libra_data(&db).await?;
    process_aggregations(&db).await?;
    Ok(())
}
