use color_eyre::eyre::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use walletconnect_rpc::{api::core::RelayClient, Client};

mod app;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    color_eyre::install()?;
    let app: app::App = argh::from_env();
    let pairing_uri = app.pairing_uri;

    let client = Client::new().await?;
    log::info!("Client instantiated with pairing_uri: {:#?}", pairing_uri);
    let val = client.inner().relay_subscribe(pairing_uri.topic.clone()).await?;
    println!("\n[client1] subscribed: topic={}", pairing_uri.topic);

    Ok(())
}

fn init_logging() {
    let fmt = fmt::layer().compact();
    Registry::default().with(EnvFilter::from_default_env()).with(fmt).init()
}
