use std::error::Error;
use backend::db::ScyllaDb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let uri = "127.0.0.1:9042";
    let scylla_db = ScyllaDb::create_session(uri).await?;
    scylla_db.initialize().await?;
    Ok(())
}
