use crate::error::Error;
use std::env;

pub fn config_env_vars() -> Result<(String, String), Error> {
    let project_id = env::var("PROJECT_ID")?;
    let collection_name = env::var("COLLECTION_NAME")?;
    Ok((project_id, collection_name))
}
