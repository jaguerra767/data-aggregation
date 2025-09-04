use std::env;

use firestore::errors::FirestoreError;
use thiserror::Error;
use warp::reject::Reject;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to load environment variables")]
    EnvError(#[from] env::VarError),
    #[error("Failed to connect to Firestore")]
    FirestoreError(#[from] FirestoreError),
    #[error("JSON serialization/deserialization error")]
    JsonError(#[from] serde_json::Error),
}
impl Reject for Error {}
