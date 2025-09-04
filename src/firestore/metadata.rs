use crate::error::Error;
use crate::processing::action::ActionAggregates;
use chrono::{DateTime, Utc};
use firestore::FirestoreDb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LastProcessed {
    #[serde(with = "firestore::serialize_as_timestamp")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub last_processed: LastProcessed,
    pub last_aggregate: ActionAggregates,
}

pub async fn fetch_metadata(db: &FirestoreDb) -> Result<Option<Metadata>, Error> {
    let metadata = db
        .fluent()
        .select()
        .by_id_in("aggregates")
        .obj::<Metadata>()
        .one("metadata")
        .await?;
    Ok(metadata)
}

pub async fn update_metadata(db: &FirestoreDb, metadata: &Metadata) -> Result<(), Error> {
    // Use insert first, then update if it fails (upsert behavior)
    let insert_result = db
        .fluent()
        .insert()
        .into("aggregates")
        .document_id("metadata")
        .object(metadata)
        .execute::<()>()
        .await;

    if insert_result.is_err() {
        // Document exists, update it
        db.fluent()
            .update()
            .in_col("aggregates")
            .document_id("metadata")
            .object(metadata)
            .execute::<()>()
            .await?;
    }
    Ok(())
}
