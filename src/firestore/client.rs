use crate::error::Error;
use crate::processing::action::aggregate_actions;
use crate::processing::category::aggregate_by_category;
use crate::processing::time::{aggregate_daily, aggregate_hourly};
use firestore::*;
use firestore::FirestoreTimestamp;
use menu::{action::Action, libra_data::LibraData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;
use time::OffsetDateTime;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct FirestoreLibraData {
    pub device: FirestoreDevice,
    pub location: String,
    pub ingredient: String,
    #[serde(rename = "dataAction")]
    pub data_action: Action,
    pub amount: f64,
    #[serde(with = "firestore::serialize_as_timestamp")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirestoreDevice {
    pub model: menu::device::Model,
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
}

impl From<LibraData> for FirestoreLibraData {
    fn from(data: LibraData) -> Self {
        // Convert time::OffsetDateTime to chrono::DateTime<Utc>
        let timestamp_unix = data.timestamp.unix_timestamp();
        let timestamp_nanos = data.timestamp.nanosecond();
        let chrono_timestamp = DateTime::from_timestamp(timestamp_unix, timestamp_nanos).unwrap();
        
        Self {
            device: FirestoreDevice {
                model: data.device.model,
                serial_number: data.device.serial_number,
            },
            location: data.location,
            ingredient: data.ingredient,
            data_action: data.data_action,
            amount: data.amount,
            timestamp: chrono_timestamp,
        }
    }
}

impl From<FirestoreLibraData> for LibraData {
    fn from(data: FirestoreLibraData) -> Self {
        // Convert chrono::DateTime<Utc> to time::OffsetDateTime
        let time_timestamp = OffsetDateTime::from_unix_timestamp(data.timestamp.timestamp())
            .unwrap()
            .replace_nanosecond(data.timestamp.timestamp_subsec_nanos())
            .unwrap();
            
        Self {
            device: menu::device::Device {
                model: data.device.model,
                serial_number: data.device.serial_number,
            },
            location: data.location,
            ingredient: data.ingredient,
            data_action: data.data_action,
            amount: data.amount,
            timestamp: time_timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LastProcessed {
    pub id: String,
    #[serde(with = "firestore::serialize_as_timestamp")]
    pub timestamp: DateTime<Utc>,
}

async fn get_last_processed(db: &FirestoreDb) -> Result<Option<LastProcessed>, Error> {
    let last_processed = db
        .fluent()
        .select()
        .by_id_in("aggregates")
        .obj::<LastProcessed>()
        .one("last_processed")
        .await?;
    Ok(last_processed)
}

async fn update_last_processed(db: &FirestoreDb, entries: &[LibraData]) -> Result<(), Error> {
    if let Some(latest_entry) = entries.iter().max_by_key(|entry| entry.timestamp) {
        // Convert time::OffsetDateTime to chrono::DateTime<Utc>
        let timestamp_unix = latest_entry.timestamp.unix_timestamp();
        let timestamp_nanos = latest_entry.timestamp.nanosecond();
        let chrono_timestamp = DateTime::from_timestamp(timestamp_unix, timestamp_nanos).unwrap();
        
        let last_processed = LastProcessed {
            id: "last_processed".to_string(),
            timestamp: chrono_timestamp,
        };
        
        // Use insert first, then update if it fails (upsert behavior)
        let insert_result = db.fluent()
            .insert()
            .into("aggregates")
            .document_id("last_processed")
            .object(&last_processed)
            .execute::<()>()
            .await;
            
        if insert_result.is_err() {
            // Document exists, update it
            db.fluent()
                .update()
                .in_col("aggregates")
                .document_id("last_processed")
                .object(&last_processed)
                .execute::<()>()
                .await?;
        }
    }
    Ok(())
}

async fn fetch_new_entries(db: &FirestoreDb) -> Result<Vec<LibraData>, Error> {
    let last_processed = get_last_processed(db).await?;
    println!("Last processed: {:?}", last_processed);
    
    // Debug: Check what data exists in Firestore (last 5 entries)
    let all_data_sample: Vec<FirestoreLibraData> = db.fluent()
        .select()
        .from("libra")
        .order_by([("timestamp", firestore::FirestoreQueryDirection::Descending)])
        .limit(5)
        .obj()
        .query()
        .await
        .unwrap_or_default();
    
    println!("Recent entries in Firestore:");
    for entry in &all_data_sample {
        println!("  Timestamp: {}", entry.timestamp);
    }
    
    let firestore_data: Vec<FirestoreLibraData> = match last_processed {
        Some(last_proc) => {
            println!("Filtering entries newer than: {}", last_proc.timestamp);
            
            // Convert DateTime<Utc> to FirestoreTimestamp
            let firestore_timestamp = FirestoreTimestamp::from(last_proc.timestamp);
            
            let result = db.fluent()
                .select()
                .from("libra")
                .filter(|q| {
                    q.field("timestamp")
                        .greater_than(firestore_timestamp.clone())
                })
                .obj()
                .query()
                .await;
            
            match result {
                Ok(data) => data,
                Err(e) => {
                    println!("Warning: Failed to fetch some documents, continuing with partial data: {}", e);
                    Vec::new()
                }
            }
        }
        None => {
            // No last_processed document exists, fetch all entries
            println!("No last_processed document found, fetching all entries");
            let result = db.fluent()
                .select()
                .from("libra")
                .obj()
                .query()
                .await;
                
            match result {
                Ok(data) => data,
                Err(e) => {
                    println!("Warning: Failed to fetch some documents, continuing with partial data: {}", e);
                    Vec::new()
                }
            }
        }
    };
    
    println!("Query returned {} entries", firestore_data.len());
    
    // Debug: Print timestamps of returned data
    for entry in &firestore_data {
        println!("Found entry with timestamp: {}", entry.timestamp);
    }
    
    // Convert back to LibraData
    let data: Vec<LibraData> = firestore_data.into_iter().map(LibraData::from).collect();
    Ok(data)
}

async fn write_by_category(
    db: &FirestoreDb,
    aggregates: HashMap<String, usize>,
) -> Result<(), Error> {
    for (category, count) in aggregates {
        db.fluent()
            .update()
            .in_col("aggregates_categories")
            .document_id(category)
            .object(&serde_json::json!({"count": count}))
            .execute::<()>()
            .await?;
    }
    Ok(())
}

async fn write_by_action(
    db: &FirestoreDb,
    aggregates: HashMap<Action, usize>,
) -> Result<(), Error> {
    for (action, count) in aggregates {
        db.fluent()
            .update()
            .in_col("aggregates_actions")
            .document_id(action.to_string())
            .object(&serde_json::json!({"count": count}))
            .execute::<()>()
            .await?;
    }
    Ok(())
}

async fn write_by_hour(db: &FirestoreDb, aggregates: HashMap<u8, usize>) -> Result<(), Error> {
    for (hour, count) in aggregates {
        db.fluent()
            .update()
            .in_col("aggregates_time_hours")
            .document_id(format!("{}", hour))
            .object(&serde_json::json!({"count": count}))
            .execute::<()>()
            .await?;
    }
    Ok(())
}

async fn write_by_date(db: &FirestoreDb, aggregates: HashMap<Date, usize>) -> Result<(), Error> {
    for (date, count) in aggregates {
        db.fluent()
            .update()
            .in_col("aggregates_time_dates")
            .document_id(date.to_string())
            .object(&serde_json::json!({"count": count}))
            .execute::<()>()
            .await?;
    }
    Ok(())
}

pub async fn process_aggregations(db: &FirestoreDb) -> Result<(), Error> {
    let entries = fetch_new_entries(&db).await?;
    println!("Fetched {} entries for processing", entries.len());
    
    if entries.is_empty() {
        println!("No entries to process");
        return Ok(());
    }

    let action_aggregates = aggregate_actions(&entries);
    println!("Action aggregates: {:?}", action_aggregates);
    write_by_action(&db, action_aggregates).await?;

    let hourly_aggregates = aggregate_hourly(&entries, Action::Served);
    println!("Hourly aggregates: {:?}", hourly_aggregates);
    write_by_hour(&db, hourly_aggregates).await?;

    let daily_aggregates = aggregate_daily(&entries, Action::Served);
    println!("Daily aggregates: {:?}", daily_aggregates);
    write_by_date(&db, daily_aggregates).await?;

    let category_aggregates = aggregate_by_category(&entries);
    println!("Category aggregates: {:?}", category_aggregates);
    write_by_category(&db, category_aggregates).await?;

    // Update the last processed timestamp
    update_last_processed(&db, &entries).await?;
    println!("Updated last processed timestamp");

    Ok(())
}
