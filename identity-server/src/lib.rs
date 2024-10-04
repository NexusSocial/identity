pub mod config;
pub mod jwk;
pub mod jwks_provider;
pub mod oauth;
pub mod v1;

mod uuid;

use std::{
	future::IntoFuture,
	net::{Ipv6Addr, SocketAddr},
	str::FromStr,
};

use axum::routing::get;
use color_eyre::{eyre::WrapErr as _, Result};
use config::TlsConfig;
use futures::FutureExt;
use sqlx::sqlite::SqlitePool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::HttpConfig;

pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Eq, PartialEq, Default)]
pub enum Env {
	#[default]
	Prod,
	Stage,
}

#[derive(thiserror::Error, Debug)]
#[error("failed to parse from env var")]
pub struct EnvParseErr;

impl FromStr for Env {
	type Err = EnvParseErr;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"stage" => Ok(Env::Stage),
			"prod" => Ok(Env::Prod),
			_ => Err(EnvParseErr),
		}
	}
}

impl Env {
	pub fn from_env() -> Self {
		std::env::var("IDENTITY_SERVER_ENV")
			.as_deref()
			.unwrap_or("prod")
			.parse::<Self>()
			.unwrap_or_default()
	}
}

/// A [`SqlitePool`] that has already been migrated.
#[derive(Debug, Clone)]
pub struct MigratedDbPool(SqlitePool);

impl MigratedDbPool {
	pub async fn new(pool: SqlitePool) -> Result<Self> {
		MIGRATOR
			.run(&pool)
			.await
			.wrap_err("failed to run migrations")?;

		Ok(Self(pool))
	}
}

#[derive(Debug)]
pub struct RouterConfig {
	pub v1: crate::v1::RouterConfig,
	pub oauth: crate::oauth::OAuthConfig,
}

impl RouterConfig {
	pub async fn build(self) -> Result<axum::Router<()>> {
		let v1 = self
			.v1
			.build()
			.await
			.wrap_err("failed to build v1 router")?;

		let oauth = self
			.oauth
			.build()
			.await
			.wrap_err("failed to build oauth router")?;

		Ok(axum::Router::new()
			.route("/", get(root))
			.nest("/api/v1", v1)
			.nest("/oauth2", oauth)
			.layer(TraceLayer::new_for_http()))
	}
}

async fn root() -> &'static str {
	"uwu hewwo this api is under constwuction"
}

/// Runs a HTTPS server on a tokio task.
pub async fn spawn_https_server(
	cfg: HttpConfig,
	router: axum::Router,
) -> Result<(
	tokio::task::JoinHandle<Result<()>>,
	tokio::sync::oneshot::Sender<()>,
)> {
	assert_ne!(
		cfg.tls,
		TlsConfig::Disable,
		"sanity: configs with disabled tls don't make sense here"
	);
	let listener = bind_listener(cfg.port).await?;
	let local_addr = listener.local_addr().unwrap();
	info!("HTTPS server listening on {local_addr}",);

	let (tx, rx) = tokio::sync::oneshot::channel();
	let task_handle = tokio::spawn(async move {
		let serve_fut = axum::serve(listener, router)
			.into_future()
			.map(|r| r.wrap_err("HTTPS server crashed"));
		tokio::select! {
			result = serve_fut => result,
			_ = rx => {
				info!("killing HTTPS server due to shutdown signal");
				Ok(())
			}
		}
	});

	Ok((task_handle, tx))
}

/// Runs a HTTP server on a tokio task.
pub async fn spawn_http_server(
	cfg: HttpConfig,
	router: axum::Router,
) -> Result<(
	tokio::task::JoinHandle<Result<()>>,
	tokio::sync::oneshot::Sender<()>,
)> {
	assert_eq!(
		cfg.tls,
		TlsConfig::Disable,
		"sanity: configs with enabled TLS don't make sense here"
	);
	let listener = bind_listener(cfg.port).await?;
	let local_addr = listener.local_addr().unwrap();
	info!("HTTP server listening on {local_addr}");

	let (tx, rx) = tokio::sync::oneshot::channel();
	let task_handle = tokio::spawn(async move {
		let serve_fut = axum::serve(listener, router)
			.into_future()
			.map(|r| r.wrap_err("HTTP server crashed"));
		tokio::select! {
			result = serve_fut => result,
			_ = rx => {
				info!("killing HTTP server due to shutdown signal");
				Ok(())
			}
		}
	});

	Ok((task_handle, tx))
}

async fn bind_listener(port: u16) -> Result<TcpListener> {
	TcpListener::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port))
		.await
		.wrap_err_with(|| format!("failed to listen to tcp on port {}", port))
}
