//! V1 of the API. This is subject to change until we commit to stability, after
//! which point any breaking changes will go in a V2 api.
//!
//! # Terminology
//! * DID: Decentralized Identifiers. The machine-readable and (probably) stable
//!   identifier for an account. Resolves to a DID Document that conctains pubkeys.
//!   Example: `did:web:did.socialvr.net:v1:a250bd28-82db-4ee5-a983-01cc756b4588`.
//! * Handle: A human readable, impermanent identifier. Handles can be changed.
//!   By default, we provide handles for all users under `handle.handle_hostname`.
//!   Example: thebutlah.socialvr.net or alice.foobar.baz.com

use std::sync::Arc;

use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Redirect},
	routing::{get, post},
};
use color_eyre::eyre::{Context as _, bail};
use jose_jwk::{Jwk, JwkSet};
use tracing::error;
use url::Host;
use uuid::Uuid;

use crate::{
	MigratedDbPool,
	handle::{Handle, InvalidHandle},
	uuid::UuidProvider,
};

#[derive(Debug, Clone)]
struct RouterState {
	uuid_provider: Arc<UuidProvider>,
	db_pool: MigratedDbPool,
	did_hostname: String,
	handle_hostname: String,
}

/// Configuration for the V1 api's router.
#[derive(Debug)]
pub struct RouterConfig {
	pub uuid_provider: UuidProvider,
	pub db_pool: MigratedDbPool,
	pub did_hostname: url::Host<String>,
	pub handle_hostname: url::Host<String>,
}

impl RouterConfig {
	pub async fn build(self) -> color_eyre::Result<Router> {
		let Host::Domain(did_hostname) = self.did_hostname else {
			bail!("ip addresses not supported");
		};
		let Host::Domain(handle_hostname) = self.handle_hostname else {
			bail!("ip addresses not supported");
		};
		Ok(Router::new()
			.route("/create/:handle", post(create))
			.route("/users/:id/did.json", get(read))
			.route("/.well-known/nexus-did", get(read_handle))
			.with_state(RouterState {
				uuid_provider: Arc::new(self.uuid_provider),
				db_pool: self.db_pool,
				did_hostname,
				handle_hostname,
			}))
	}
}

#[derive(thiserror::Error, Debug)]
enum CreateErr {
	#[error(transparent)]
	Internal(#[from] color_eyre::Report),
	#[error("invalid handle: {0}")]
	InvalidHandle(#[from] InvalidHandle),
	#[error("that handle is already taken")]
	HandleTaken,
	#[error("that handle is reserved")]
	HandleReserved,
	#[error(
		"handle contained a dot, which is only valid for handles on third-party domains"
	)]
	HandleContainedDot,
}

impl IntoResponse for CreateErr {
	fn into_response(self) -> axum::response::Response {
		error!("{self:?}");
		match self {
			Self::Internal(_) => {
				(StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
			}
			Self::InvalidHandle(_) => {
				(StatusCode::BAD_REQUEST, self.to_string()).into_response()
			}
			Self::HandleTaken => {
				(StatusCode::CONFLICT, self.to_string()).into_response()
			}
			Self::HandleReserved => {
				(StatusCode::FORBIDDEN, self.to_string()).into_response()
			}
			Self::HandleContainedDot => {
				(StatusCode::FORBIDDEN, self.to_string()).into_response()
			}
		}
	}
}

#[tracing::instrument(skip_all)]
async fn create(
	state: State<RouterState>,
	handle: Path<String>,
	pubkey: Json<Jwk>,
) -> Result<Redirect, CreateErr> {
	let handle: Handle = handle.parse()?;

	let handle_to_store =
		if let Some(prefix) = handle.as_str().strip_suffix(&state.handle_hostname) {
			// handle on our domain
			let prefix = prefix.strip_suffix(".").expect("infallible");
			if crate::handle::is_handle_prefix_reserved(prefix) {
				return Err(CreateErr::HandleReserved);
			}
			if prefix.contains('.') {
				return Err(CreateErr::HandleContainedDot);
			}
			prefix
		} else {
			// handle not on our domain
			handle.as_str()
		};

	let uuid = state.uuid_provider.next_v4();
	let jwks = JwkSet {
		keys: vec![pubkey.0],
	};
	let serialized_jwks = serde_json::to_string(&jwks).expect("infallible");

	sqlx::query(
		"INSERT INTO users (user_id, handle, pubkeys_jwks) VALUES ($1, $2, $3)",
	)
	.bind(uuid)
	.bind(handle_to_store)
	.bind(serialized_jwks)
	.execute(&state.db_pool.0)
	.await
	.inspect_err(|err| error!(?err, "error while inserting new account into DB"))
	.map_err(|_| CreateErr::HandleTaken)?;

	Ok(Redirect::to(&format!(
		"/users/{}/did.json",
		uuid.as_hyphenated()
	)))
}

#[derive(thiserror::Error, Debug)]
enum ReadErr {
	#[error("no such user exists")]
	NoSuchUser,
	#[error(transparent)]
	Internal(#[from] color_eyre::Report),
}

impl IntoResponse for ReadErr {
	fn into_response(self) -> axum::response::Response {
		error!("{self:?}");
		match self {
			Self::NoSuchUser => {
				(StatusCode::NOT_FOUND, self.to_string()).into_response()
			}
			Self::Internal(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
			}
		}
	}
}

// TODO: currently this returns a JSON Web Key Set, but we actually want to be
// returning a did:web json.
#[tracing::instrument(skip_all)]
async fn read(
	state: State<RouterState>,
	Path(user_id): Path<Uuid>,
) -> Result<Json<JwkSet>, ReadErr> {
	let keyset_in_string: Option<String> =
		sqlx::query_scalar("SELECT pubkeys_jwks FROM users WHERE user_id = $1")
			.bind(user_id)
			.fetch_optional(&state.db_pool.0)
			.await
			.wrap_err("failed to retrieve from database")?;
	let Some(keyset_in_string) = keyset_in_string else {
		return Err(ReadErr::NoSuchUser);
	};
	// TODO: Do we actually care about round-trip validating the JwkSet here?
	let keyset: JwkSet = serde_json::from_str(&keyset_in_string)
		.wrap_err("failed to deserialize JwkSet from database")?;

	Ok(Json(keyset))
}

#[derive(thiserror::Error, Debug)]
enum ReadHandleErr {
	#[error("no such handle exists")]
	NoSuchHandle,
	#[error("wrong hostname")]
	UnexpectedHostname,
	#[error(transparent)]
	Internal(#[from] color_eyre::Report),
}

impl IntoResponse for ReadHandleErr {
	fn into_response(self) -> axum::response::Response {
		error!("{self:?}");
		match self {
			Self::UnexpectedHostname => {
				(StatusCode::MISDIRECTED_REQUEST, self.to_string()).into_response()
			}
			Self::NoSuchHandle => {
				(StatusCode::NOT_FOUND, self.to_string()).into_response()
			}
			Self::Internal(err) => {
				(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
			}
		}
	}
}

async fn read_handle(
	host: axum::extract::Host,
	state: State<RouterState>,
) -> Result<String, ReadHandleErr> {
	let Some(handle_prefix) = host
		.0
		.strip_suffix(&state.handle_hostname)
		.and_then(|p| p.strip_suffix("."))
	else {
		return Err(ReadHandleErr::UnexpectedHostname);
	};

	let uuid: Option<Uuid> =
		sqlx::query_scalar("SELECT user_id FROM users WHERE handle = $1")
			.bind(handle_prefix)
			.fetch_optional(&state.db_pool.0)
			.await
			.wrap_err("failed to retrieve from database")?;
	let Some(uuid) = uuid else {
		return Err(ReadHandleErr::NoSuchHandle);
	};

	let did = crate::did::uuid_to_did(&state.did_hostname, &uuid);
	Ok(did)
}

#[cfg(test)]
mod tests {
	use crate::jwk::ed25519_pub_jwk;

	use super::*;
	use axum::{
		body::Body,
		http::{self, Request, Response},
	};
	use color_eyre::Result;
	use http_body_util::BodyExt;
	use jose_jwk::OkpCurves;
	use sqlx::{Row, SqlitePool};
	use tower::ServiceExt as _; // for `collect`

	fn uuids(range: std::ops::Range<u128>) -> Vec<Uuid> {
		range.map(Uuid::from_u128).collect()
	}

	const TEST_ROUTER_UUID_START: u128 = 100;

	/// Creates a router for testing purposes.
	async fn test_router(db_pool: &SqlitePool, hostname: &str) -> Router {
		let db_pool = crate::MigratedDbPool::new(db_pool.clone())
			.await
			.expect("failed to migrate db");

		let router = RouterConfig {
			uuid_provider: UuidProvider::new_from_sequence(uuids(
				TEST_ROUTER_UUID_START..(TEST_ROUTER_UUID_START + 10),
			)),
			db_pool,
			did_hostname: url::Host::parse(&format!("did.{hostname}")).unwrap(),
			handle_hostname: url::Host::parse(hostname).unwrap(),
		};
		router.build().await.expect("failed to build router")
	}

	/// Creates a dummy pubkey.
	fn dummy_key() -> did_simple::crypto::ed25519::VerifyingKey {
		did_simple::crypto::ed25519::SigningKey::from_bytes(
			&[0; ed25519_dalek::SECRET_KEY_LENGTH],
		)
		.verifying_key()
	}

	/// Prints the contents of the database.
	async fn print_db(db_pool: &SqlitePool) {
		let rows = sqlx::query("SELECT * FROM users")
			.fetch_all(db_pool)
			.await
			.expect("failed to read database");
		println!("{{");
		for r in rows {
			let n_columns = r.len();
			for c in 0..n_columns {
				if let Ok(s) = r.try_get::<String, _>(c) {
					print!("{s}, ");
				} else if let Ok(uuid) = r.try_get::<Uuid, _>(c) {
					print!("{uuid}, ");
				} else if let Ok(bytes) = r.try_get::<Vec<u8>, _>(c) {
					print!("{bytes:?}, ");
				} else {
					unimplemented!("no other datatypes needed");
				}
			}
			println!();
		}
		println!("}}");
	}

	/// Performs HTTP GET to read a user handle.
	async fn request_read_handle(router: Router, handle: &str) -> Response<Body> {
		let req = Request::builder()
			.method("GET")
			.uri(format!("https://{handle}/.well-known/nexus-did"))
			.body(axum::body::Body::empty())
			.unwrap();

		router.oneshot(req).await.unwrap()
	}

	/// Performs HTTP POST to create a user.
	async fn request_create_user(
		router: Router,
		handle: &str,
		key: &did_simple::crypto::ed25519::VerifyingKey,
	) -> Response<Body> {
		let req = Request::builder()
			.method("POST")
			.uri(format!("/create/{handle}"))
			.header(http::header::CONTENT_TYPE, "application/json")
			.body(axum::body::Body::from(
				serde_json::to_vec(&ed25519_pub_jwk(key)).unwrap(),
			))
			.unwrap();
		router.clone().oneshot(req).await.unwrap()
	}

	/// Performs HTTP GET to read a user's DID.
	async fn request_read_did(router: Router, user_id: Uuid) -> Response<Body> {
		let req = Request::builder()
			.method("GET")
			.uri(format!("/users/{}/did.json", user_id.as_hyphenated()))
			.body(axum::body::Body::empty())
			.unwrap();
		router.oneshot(req).await.unwrap()
	}

	/// Validates the response and ensures it matches `expected_keys`
	async fn check_response_keys(
		response: Response<Body>,
		mut expected_keys: Vec<[u8; 32]>,
	) -> Result<()> {
		assert_eq!(response.status(), StatusCode::OK);
		assert_eq!(response.headers()["Content-Type"], "application/json");
		let body = response.into_body().collect().await?.to_bytes();
		let jwks: JwkSet =
			serde_json::from_slice(&body).wrap_err("failed to deserialize response")?;
		let mut ed25519_keys: Vec<[u8; 32]> = jwks
			.keys
			.into_iter()
			.map(|jwk| {
				let jose_jwk::Key::Okp(ref key) = jwk.key else {
					panic!("did not encounter okp key group");
				};
				assert_eq!(key.crv, OkpCurves::Ed25519);
				assert!(key.d.is_none(), "private keys should not be stored");
				let key: [u8; 32] =
					key.x.as_ref().try_into().expect("wrong key length");
				key
			})
			.collect();

		ed25519_keys.sort();
		expected_keys.sort();
		assert_eq!(ed25519_keys, expected_keys);

		Ok(())
	}

	async fn check_response_handle(
		response: Response<Body>,
		expected: &str,
	) -> Result<()> {
		assert_eq!(response.status(), axum::http::StatusCode::OK);
		assert_eq!(
			response.headers()["Content-Type"],
			"text/plain; charset=utf-8"
		);
		let body = response.into_body().collect().await?.to_bytes();
		let body = String::from_utf8(body.to_vec()).expect("should be utf-8");
		assert_eq!(body, expected);

		Ok(())
	}

	/// Puts `num` as last byte of pubkey, everything else zero.
	fn key_from_number(num: u8) -> [u8; 32] {
		let mut expected_key = [0; 32];
		*expected_key.last_mut().unwrap() = num;
		expected_key
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_read_db(db_pool: SqlitePool) -> Result<()> {
		let router = test_router(&db_pool, "doesnt.matter").await;
		let response = request_read_did(router, Uuid::from_u128(1)).await;

		check_response_keys(response, vec![key_from_number(1)]).await
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_read_nonexistent_user(db_pool: SqlitePool) {
		let router = test_router(&db_pool, "doesnt.matter").await;
		let response = request_read_did(router, Uuid::nil()).await;
		assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_read_existant_handle(db_pool: SqlitePool) -> Result<()> {
		let router = test_router(&db_pool, "testhostname.com").await;
		let response = request_read_handle(router, "alice.testhostname.com").await;
		check_response_handle(
			response,
			&format!(
				"did:web:did.testhostname.com:v1:{}",
				Uuid::from_u128(1).as_hyphenated()
			),
		)
		.await
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_read_nonexistant_handle(db_pool: SqlitePool) {
		let router = test_router(&db_pool, "testhostname.com").await;
		let response =
			request_read_handle(router, "doesntexist.testhostname.com").await;
		assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_read_handle_for_other_domain(db_pool: SqlitePool) {
		let router = test_router(&db_pool, "testhostname.com").await;
		let response = request_read_handle(router, "alice.otherdomain.com").await;
		assert_eq!(
			response.status(),
			axum::http::StatusCode::MISDIRECTED_REQUEST
		);
	}

	/// Helper code that is used in some of the tests to reduce boilerplate for
	/// creating a user
	async fn create_user_test_helper(
		db_pool: SqlitePool,
		server_hostname: &str,
		user_handle: &str,
		expected_user_uuid: Uuid,
	) -> Result<()> {
		let router = test_router(&db_pool, server_hostname).await;
		print_db(&db_pool).await;

		let key = dummy_key();
		let response = request_create_user(router.clone(), user_handle, &key).await;
		print_db(&db_pool).await;
		let redirect_url = response.headers()[http::header::LOCATION].to_str().unwrap();
		assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
		assert_eq!(
			redirect_url,
			format!("/users/{}/did.json", expected_user_uuid.as_hyphenated())
		);

		// check that subsequent read of user data matches what we would expect
		let response = request_read_did(router.clone(), expected_user_uuid).await;
		check_response_keys(response, vec![key.as_inner().to_bytes()]).await?;

		// check that the handle is as expected
		let response = request_read_handle(router, user_handle).await;
		if user_handle.split_once(".").unwrap().1 == server_hostname {
			// User is on our domain so it should be a success
			check_response_handle(
				response,
				&format!(
					"did:web:did.example.com:v1:{}",
					expected_user_uuid.as_hyphenated()
				),
			)
			.await?;
		} else {
			// user is not on our domain so reading the handle should be a failure
			assert_eq!(
				response.status(),
				axum::http::StatusCode::MISDIRECTED_REQUEST
			);
		}

		Ok(())
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_create_user_in_empty_database_same_domain(
		db_pool: SqlitePool,
	) -> Result<()> {
		create_user_test_helper(
			db_pool.clone(),
			"example.com",
			"alice.example.com",
			Uuid::from_u128(TEST_ROUTER_UUID_START),
		)
		.await
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_create_user_in_empty_database_different_domain(
		db_pool: SqlitePool,
	) -> Result<()> {
		create_user_test_helper(
			db_pool.clone(),
			"server.com",
			"alice.other.com",
			Uuid::from_u128(TEST_ROUTER_UUID_START),
		)
		.await
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_create_user_in_populated_database_same_domain(
		db_pool: SqlitePool,
	) -> Result<()> {
		create_user_test_helper(
			db_pool.clone(),
			"example.com",
			"bob.example.com",
			Uuid::from_u128(TEST_ROUTER_UUID_START),
		)
		.await
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_create_user_in_populated_database_different_domain(
		db_pool: SqlitePool,
	) -> Result<()> {
		create_user_test_helper(
			db_pool.clone(),
			"server.com",
			"alice.other.com",
			Uuid::from_u128(TEST_ROUTER_UUID_START),
		)
		.await
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_create_user_fails_when_conflicting_with_existing_user_handle(
		db_pool: SqlitePool,
	) {
		let router = test_router(&db_pool, "example.com").await;
		// Note that alice is on same domain as did:web server and conflicts with
		// existing db user
		let response =
			request_create_user(router, "alice.example.com", &dummy_key()).await;
		assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);
	}

	#[sqlx::test(
		migrator = "crate::MIGRATOR",
		fixtures("../../fixtures/sample_users.sql")
	)]
	async fn test_create_user_fails_when_handle_is_reserved(db_pool: SqlitePool) {
		let router = test_router(&db_pool, "example.com").await;
		let response =
			request_create_user(router, "did.example.com", &dummy_key()).await;
		assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_create_user_succeeds_when_handle_is_reserved_but_on_different_domain(
		db_pool: SqlitePool,
	) -> Result<()> {
		create_user_test_helper(
			db_pool,
			"server.com",
			"did.otherdomain.com",
			Uuid::from_u128(TEST_ROUTER_UUID_START),
		)
		.await
	}

	#[sqlx::test(migrator = "crate::MIGRATOR")]
	async fn test_create_user_fails_when_dot_and_on_server_domain(db_pool: SqlitePool) {
		let router = test_router(&db_pool, "example.com").await;
		let response =
			request_create_user(router, "foo.bar.example.com", &dummy_key()).await;
		assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
	}
}
