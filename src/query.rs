use serde::Deserialize;

#[derive(Deserialize)]
pub struct LocationQuery {
    pub location: Option<String>,
    pub serial_number: Option<String>,
}
