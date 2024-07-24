use std::io::Result;

use backend::app::Application;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let application = Application::build("127.0.0.1", "8080").await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://127.0.0.1:{}/", application.port());
    application.run_until_stopped().await?;
    Ok(())
}
