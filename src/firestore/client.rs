use crate::error::Error;
use crate::firestore::metadata::{fetch_metadata, update_metadata, LastProcessed, Metadata};
use crate::processing::action::{aggregate_actions, ActionAggregates};
use crate::processing::time::{aggregate_daily, aggregate_hourly};
use crate::processing::category::{aggregate_by_category};
use chrono::{DateTime, Utc};
use firestore::errors::FirestoreError;
use firestore::FirestoreTimestamp;
use firestore::*;
use menu::{action::Action, libra_data::LibraData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;
use time::OffsetDateTime;

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

async fn fetch_all_entries(db: &FirestoreDb) -> Result<Vec<LibraData>, FirestoreError> {
    db.fluent().select().from("libra").obj().query().await
}

async fn fetch_new_entries(
    db: &FirestoreDb,
    last_processed: LastProcessed,
) -> Result<Vec<LibraData>, FirestoreError> {
    let timestamp = FirestoreTimestamp::from(last_processed.timestamp);
    db.fluent()
        .select()
        .from("libra")
        .filter(|q| q.field("timestamp").greater_than(timestamp.clone()))
        .obj()
        .query()
        .await
}

async fn write_by_category(
    db: &FirestoreDb,
    aggregates: &HashMap<String, usize>,
) -> Result<(), Error> {
    db.fluent()
        .update()
        .in_col("aggregates")
        .document_id("categories")
        .object(aggregates)
        .execute::<()>()
        .await?;

    Ok(())
}

async fn fetch_by_category(
    db: &FirestoreDb,
) -> Result<Option<HashMap<String, usize>>, FirestoreError> {
    db.fluent()
        .select()
        .by_id_in("aggregates")
        .obj::<HashMap<String, usize>>()
        .one("category")
        .await
}

async fn write_by_action(db: &FirestoreDb, aggregates: &ActionAggregates) -> Result<(), Error> {
    db.fluent()
        .update()
        .in_col("aggregates")
        .document_id("actions")
        .object(aggregates)
        .execute::<()>()
        .await?;
    Ok(())
}

async fn write_by_hour(db: &FirestoreDb, aggregates: &HashMap<u8, usize>) -> Result<(), Error> {
    db.fluent()
        .update()
        .in_col("aggregates")
        .document_id("hourly")
        .object(aggregates)
        .execute::<()>()
        .await?;

    Ok(())
}

async fn fetch_hourly_aggregates(
    db: &FirestoreDb,
) -> Result<Option<HashMap<u8, usize>>, FirestoreError> {
    db.fluent()
        .select()
        .by_id_in("aggregates")
        .obj::<HashMap<u8, usize>>()
        .one("hourly")
        .await
}

async fn write_by_date(db: &FirestoreDb, aggregates: &HashMap<Date, usize>) -> Result<(), Error> {
    db.fluent()
        .update()
        .in_col("aggregates")
        .document_id("daily")
        .object(aggregates)
        .execute::<()>()
        .await?;
    Ok(())
}

async fn fetch_daily_aggregates(
    db: &FirestoreDb,
) -> Result<Option<HashMap<Date, usize>>, FirestoreError> {
    db.fluent()
        .select()
        .by_id_in("aggregates")
        .obj::<HashMap<Date, usize>>()
        .one("hourly")
        .await
}

pub async fn process_aggregations(db: &FirestoreDb) -> Result<(), Error> {
    let (entries, last_aggregate) = match fetch_metadata(db).await? {
        Some(metadata) => (
            fetch_new_entries(db, metadata.last_processed).await?,
            metadata.last_aggregate,
        ),
        None => (fetch_all_entries(db).await?, ActionAggregates::new()),
    };

    println!("Fetched {} entries for processing", entries.len());

    if entries.is_empty() {
        println!("No entries to process");
        return Ok(());
    }

    let action_aggregates = aggregate_actions(entries.as_slice(), &last_aggregate);
    write_by_action(db, &action_aggregates).await?;

    if let Some(agg) = fetch_hourly_aggregates(db).await? {
        let hourly_aggregates = aggregate_hourly(&entries, Action::Served, &agg);
        write_by_hour(db, &hourly_aggregates).await?;
    }

    if let Some(agg) = fetch_daily_aggregates(db).await? {
        let daily_aggregates = aggregate_daily(&entries, Action::Served, &agg);
        write_by_date(db, &daily_aggregates).await?;
    }
    
    if let Some(agg) = fetch_by_category(db).await? {
        let category_aggregates = aggregate_by_category(&entries, &agg);
        write_by_category(db, &category_aggregates).await?;
    }


    // Update the last processed timestamp
    let metadata = Metadata {
        last_processed: LastProcessed {
            timestamp: Utc::now(),
        },
        last_aggregate: action_aggregates.clone(),
    };
    update_metadata(db, &metadata).await?;
    println!("Updated last processed timestamp");

    Ok(())
}
