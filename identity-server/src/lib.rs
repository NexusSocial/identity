pub mod config;
pub mod jwk;
pub mod jwks_provider;
pub mod oauth;
pub mod v1;

mod uuid;

use std::{
	future::IntoFuture,
	net::{Ipv6Addr, SocketAddr},
	num::NonZeroU16,
	str::FromStr,
};

use axum::{
	extract::Host,
	handler::HandlerWithoutStateExt as _,
	http::{uri::Authority, Uri},
	response::Redirect,
	routing::get,
};
use color_eyre::{
	eyre::{Report, WrapErr as _},
	Result,
};
use futures::FutureExt;
use sqlx::sqlite::SqlitePool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::{HttpConfig, HttpsConfig};

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

/// Spawns a tokio task for handling HTTP requests and redirecting them to their https
/// counterpart.
pub async fn spawn_http_redirect_server(
	http_port: u16,
	https_port: NonZeroU16,
) -> Result<(
	tokio::task::JoinHandle<color_eyre::Result<()>>,
	tokio::sync::oneshot::Sender<()>,
	NonZeroU16,
)> {
	let listener = bind_listener(http_port).await?;
	let local_addr = listener.local_addr().unwrap();
	info!("HTTP redirect server listening on {local_addr}",);

	let redirect = move |Host(host): Host, uri: Uri| async move {
		match remap_http_to_https_url(&host, uri, https_port) {
			Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
			Err(error) => {
				tracing::warn!(%error, "failed to convert URI to HTTPS");
				Err(axum::http::StatusCode::BAD_REQUEST)
			}
		}
	};
	let serve_fut = axum::serve(listener, redirect.into_make_service())
		.into_future()
		.map(|r| r.wrap_err("HTTP redirect server crashed"));

	let (tx, rx) = tokio::sync::oneshot::channel();
	let task_handle = tokio::spawn(async move {
		tokio::select! {
			result = serve_fut => result,
			_ = rx => {
				info!("killing HTTP redirect server due to shutdown signal");
				Ok(())
			}
		}
	});
	Ok((task_handle, tx, local_addr.port().try_into().unwrap()))
}

/// Runs a real https server on a tokio task. Also returns the port it bound to.
pub async fn spawn_https_server(
	cfg: HttpsConfig,
	router: axum::Router,
) -> Result<(
	tokio::task::JoinHandle<Result<()>>,
	tokio::sync::oneshot::Sender<()>,
	NonZeroU16,
)> {
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

	Ok((task_handle, tx, local_addr.port().try_into().unwrap()))
}

/// Runs a real http server on a tokio task.
pub async fn spawn_http_server(
	cfg: HttpConfig,
	router: axum::Router,
) -> Result<(
	tokio::task::JoinHandle<Result<()>>,
	tokio::sync::oneshot::Sender<()>,
	NonZeroU16,
)> {
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

	Ok((task_handle, tx, local_addr.port().try_into().unwrap()))
}

async fn bind_listener(port: u16) -> Result<TcpListener> {
	TcpListener::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port))
		.await
		.wrap_err_with(|| format!("failed to listen to tcp on port {}", port))
}

/// Port should already be known, which is why the port number is nonzero.
fn remap_http_to_https_url(
	host: &str,
	uri: Uri,
	https_port: NonZeroU16,
) -> Result<Uri, Report> {
	let mut parts = uri.into_parts();

	parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

	if parts.path_and_query.is_none() {
		parts.path_and_query = Some("/".parse().unwrap());
	}

	// From: https://github.com/tokio-rs/axum/pull/2792
	let authority: Authority = host.parse().wrap_err("invalid host")?;
	let bare_host: &str = match authority.port() {
            Some(port) => authority
                .as_str()
                .strip_suffix(port.as_str())
                .unwrap()
                .strip_suffix(':')
                .expect("should be infallible, all valid `Authority` structs should satisfy this"),
            None => authority.as_str(),
        };

	// TODO: reuse format buffer to avoid reallocating
	parts.authority = Some(format!("{bare_host}:{https_port}").parse()?);

	Ok(Uri::from_parts(parts)?)
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use axum::http::Uri;

	use crate::remap_http_to_https_url;

	#[test]
	fn test_http_url_maps_to_https() {
		let new_uri = remap_http_to_https_url(
			"foo.example.com:1337",
			Uri::from_str("http://foo.example.com:1337/some/path").unwrap(),
			443.try_into().unwrap(),
		)
		.expect("remap should not error");
		assert_eq!(
			new_uri,
			Uri::from_str("https://foo.example.com:443/some/path").unwrap(),
			"the new uri did not match the expected uri",
		)
	}
}
