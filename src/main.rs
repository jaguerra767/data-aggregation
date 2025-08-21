use data_aggregation::error::Error;
use dotenv::dotenv;
use firestore::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let project = env::var("GOOGLE_CLOUD_PROJECT")?;
    let _db = FirestoreDb::new(project.to_string()).await?;

    // let test = LibraData{
    //     device: Device {
    //         model: Model::LibraV0,
    //         serial_number: "kwek;a".to_string()
    //     },
    //     location: "deez".to_string(),
    //     ingredient: "nutz".to_string(),
    //     data_action: Action::Served,
    //     amount: 0.0,
    //     timestamp: OffsetDateTime::now_local().unwrap()
    // };

    //println!("Hello, world! data: {:?}", test);
    Ok(())
}
