use crate::error::Error;
use crate::error::Error::FirestoreError;
use crate::firestore::client::FirestoreLibraData;
use firestore::{FirestoreDb, FirestoreQueryDirection};
use menu::action::Action;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct LocationQuery {
    pub location: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Deserialize, Debug)]
pub enum OrderBy {
    Descending,
    Ascending,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DataQuery {
    pub location: Option<String>,
    pub serial_number: Option<String>,
    pub ingredient: Option<String>,
    pub action: Option<Action>,
    pub order_by: Option<OrderBy>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<usize>,
}
impl DataQuery {
    pub async fn run_query(&self, db: &FirestoreDb) -> Result<Vec<FirestoreLibraData>, Error> {
        let mut query = db
            .fluent()
            .select()
            .from("libra")
            .filter(|q| {
                q.for_all([
                    self.location
                        .clone()
                        .and_then(|v| q.field("location").eq(v)),
                    self.serial_number
                        .clone()
                        .and_then(|v| q.field("serial_number").eq(v)),
                    self.ingredient
                        .clone()
                        .and_then(|v| q.field("ingredient").eq(v)),
                    self.action.clone().and_then(|v| q.field("action").eq(v)),
                    self.start_date
                        .and_then(|v| q.field("timestamp").greater_than_or_equal(v)),
                    self.end_date
                        .and_then(|v| q.field("timestamp").less_than_or_equal(v)),
                ])
            })
            .order_by([(
                "timestamp",
                match &self.order_by {
                    Some(OrderBy::Ascending) => FirestoreQueryDirection::Ascending,
                    _ => FirestoreQueryDirection::Descending,
                },
            )]);
        if let Some(limit) = self.limit {
            query = query.limit(limit as u32)
        }
        let documents = query.obj::<Value>().query().await.map_err(FirestoreError)?;

        let valid_data = documents
            .iter()
            .filter_map(|data| {
                if let Ok(value) = serde_json::from_value::<FirestoreLibraData>(data.clone()) {
                    Some(value)
                } else {
                    eprintln!("Ignoring invalid data schema: {data}");
                    None
                }
            })
            .collect();
        Ok(valid_data)
    }
}
