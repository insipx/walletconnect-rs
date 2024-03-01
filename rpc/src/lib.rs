mod api;
pub mod auth;
pub mod error;
pub mod types;

// better-walletconnect-rs
pub const PROJECT_ID: &str = "c391bf7391b67ffbd8b8241389471ef8";

#[cfg(test)]
mod test {
    use std::sync::Once;
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
    };

    static INIT: Once = Once::new();

    #[ctor::ctor]
    fn __init_test_logging() {
        INIT.call_once(|| {
            let fmt = fmt::layer().compact();
            Registry::default()
                .with(EnvFilter::from_default_env())
                .with(fmt)
                .init()
        })
    }
}
