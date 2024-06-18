use std::io::Result;

use backend::{ app::Application, config::get_config };

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config = get_config().expect("Failed to read config");
    let application = Application::build(config).await?;

    tracing::event!(target: "backend", tracing::Level::INFO, "Listening on http://127.0.0.1:{}/", application.port());
    application.run_until_stopped().await?;
    Ok(())
}
