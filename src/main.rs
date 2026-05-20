 mod app;
 
 use anyhow::Context;
 use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
 
 #[tokio::main]
 async fn main() -> anyhow::Result<()> {
     dotenvy::dotenv().ok();
 
     tracing_subscriber::registry()
         .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
             "ngn_verify=debug,tower_http=info,axum=info".into()
         }))
         .with(tracing_subscriber::fmt::layer())
         .init();
 
     let config = app::config::Config::from_env().context("load config")?;
     let state = app::state::AppState::new(&config).await.context("init app state")?;
 
     let app = app::router::build_router(state, config.cors_allow_any_origin);
 
     let addr = std::net::SocketAddr::new(config.host, config.port);
     tracing::info!(%addr, "listening");
 
     let listener = tokio::net::TcpListener::bind(addr).await?;
     axum::serve(listener, app)
         .with_graceful_shutdown(shutdown_signal())
         .await?;
 
     Ok(())
 }
 
 async fn shutdown_signal() {
     let ctrl_c = async {
         tokio::signal::ctrl_c()
             .await
             .expect("failed to install Ctrl+C handler");
     };
 
     #[cfg(unix)]
     let terminate = async {
         tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
             .expect("failed to install signal handler")
             .recv()
             .await;
     };
 
     #[cfg(not(unix))]
     let terminate = std::future::pending::<()>();
 
     tokio::select! {
         _ = ctrl_c => {},
         _ = terminate => {},
     }
 
     tracing::info!("shutdown signal received");
 }
