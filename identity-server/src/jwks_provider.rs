use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use async_trait::async_trait;
use color_eyre::{Result, Section, eyre::WrapErr as _};
use jsonwebtoken::jwk::JwkSet;
use reqwest::Url;
use tracing::{debug, info};

/// Retrieves the latest JWKs for an external service.
///
/// Example: This can be used to get the JWKs from google, located at
/// <https://www.googleapis.com/oauth2/v3/certs>
///
/// This provider exists to support mocking of the external interface, for the purposes
/// of testing.
#[derive(Debug)]
pub struct JwksProvider {
	#[cfg(not(test))]
	provider: HttpProvider,
	#[cfg(test)]
	provider: Box<dyn JwksProviderT>,
}

impl JwksProvider {
	pub fn google(client: reqwest::Client) -> Self {
		Self {
			#[cfg(not(test))]
			provider: HttpProvider::google(client),
			#[cfg(test)]
			provider: Box::new(HttpProvider::google(client)),
		}
	}

	pub async fn get(&self) -> Result<Arc<CachedJwks>> {
		self.provider.get().await
	}
}

#[async_trait]
trait JwksProviderT: std::fmt::Debug + Send + Sync + 'static {
	/// Gets the latest Json Web Key Set.
	async fn get(&self) -> Result<Arc<CachedJwks>>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct CachedJwks {
	jwks: JwkSet,
	expires_at: std::time::Instant,
}

impl CachedJwks {
	/// Creates an empty set of JWKs, which is already expired.
	fn new_expired() -> Self {
		let now = std::time::Instant::now();
		let expires_at = now.checked_sub(Duration::from_secs(1)).unwrap_or(now);
		Self {
			jwks: JwkSet { keys: vec![] },
			expires_at,
		}
	}

	pub fn jwks(&self) -> &JwkSet {
		&self.jwks
	}

	fn is_expired(&self) -> bool {
		self.expires_at <= std::time::Instant::now()
	}
}

/// Uses http to retrieve the JWKs.
#[derive(Debug)]
struct HttpProvider {
	url: Url,
	client: reqwest::Client,
	cached_jwks: ArcSwap<CachedJwks>,
}

impl HttpProvider {
	pub fn new(url: Url, client: reqwest::Client) -> Self {
		// Creates immediately expired empty keyset
		Self {
			client,
			url,
			cached_jwks: ArcSwap::new(Arc::new(CachedJwks::new_expired())),
		}
	}

	/// Creates a provider that requests the JWKS over HTTP from google's url.
	pub fn google(client: reqwest::Client) -> Self {
		Self::new(
			"https://www.googleapis.com/oauth2/v3/certs"
				.try_into()
				.unwrap(),
			client,
		)
	}
}

#[async_trait]
impl JwksProviderT for HttpProvider {
	/// Usually this is instantly ready with the JWKS, but if the cached value doesn't
	/// exist
	/// or is out of date, it will await on the new value.
	async fn get(&self) -> Result<Arc<CachedJwks>> {
		let cached_jwks = self.cached_jwks.load();
		if !cached_jwks.is_expired() {
			return Ok(cached_jwks.to_owned());
		}
		let response = self
			.client
			.get(self.url.clone())
			.send()
			.await
			.wrap_err("failed to initiate get request for certs")
			.and_then(|resp| {
				resp.error_for_status()
					.wrap_err("request for certs returned HTTP error code")
			})
			.with_note(|| format!("url was {}", self.url))?;

		let expires_at = {
			if let Some(duration) =
				header_parsing::time_until_max_age(response.headers())
			{
				std::time::Instant::now() + duration
			} else {
				std::time::Instant::now()
			}
		};
		let serialized_keys = response
			.bytes()
			.await
			.wrap_err("failed to get response body")?;
		debug!(body = ?serialized_keys, "got response body");
		let jwks: JwkSet = serde_json::from_slice(&serialized_keys)
			.wrap_err("unexpected response, expected a JWKS")?;
		let cached_jwks = Arc::new(CachedJwks { jwks, expires_at });
		self.cached_jwks.store(Arc::clone(&cached_jwks));
		info!("cached JWKs: {cached_jwks:?}");
		Ok(cached_jwks)
	}
}

/// Always provides the same JWKs.
#[derive(Debug, Clone)]
#[expect(dead_code)]
struct StaticProvider(Arc<CachedJwks>);

#[async_trait]
impl JwksProviderT for StaticProvider {
	async fn get(&self) -> Result<Arc<CachedJwks>> {
		Ok(Arc::clone(&self.0))
	}
}

#[cfg(test)]
mod test {
	use std::sync::OnceLock;

	use super::*;
	use axum::http::header::{AGE, CACHE_CONTROL};
	use tracing_test::traced_test;
	use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

	fn client() -> &'static reqwest::Client {
		static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
		CLIENT.get_or_init(|| {
			reqwest::Client::builder()
				.timeout(Duration::from_secs(1))
				.build()
				.unwrap()
		})
	}

	fn example_jwks() -> &'static JwkSet {
		static JWKS: OnceLock<JwkSet> = OnceLock::new();
		JWKS.get_or_init(|| {

		serde_json::from_value(serde_json::json!({
		  "keys": [
			{
			  "e": "AQAB",
			  "alg": "RS256",
			  "kty": "RSA",
			  "use": "sig",
			  "n": "jPxgqe78Uy8UI0nrbys8zFQnskdLnvY9DFAKbI9Or7sPc7vhyQ-ynHWXrvrv3J3EVqcqwZSTAjiKbSbIhKRF2iXyIP5jmhS6QTUQb7D8smC89yZi6Ii-AzpH6QKvmhU7yJ1u0odMM1UDUS5bH5aL50HxxqqaQGlZ7PFOT0xrauAFW-3ONVc7_tXGMbfYRzeRrXqaONJ1B9LOconUlsBsL0U1TepINyztbwjM3NBlvEuBX0m4ZbCFznGoWmnix3FuUS4gAybOO3WYr6Zd71cKBFPfdpMMfNjWM2pf1-1O1IF8iArGbvngn8Vk5QGH3MkJDA_JgZOu9pI64LSIEKG02w",
			  "kid": "5aaff47c21d06e266cce395b2145c7c6d4730ea5"
			},
			{
			  "alg": "RS256",
			  "use": "sig",
			  "kid": "28a421cafbe3dd889271df900f4bbf16db5c24d4",
			  "n": "1BqxSPBr-Fap-E39TLXfuDg0Bfg05zYqhvVvEVhfPXRkPj7M8uK_1MOb-11XKaZ4IkWMJIwRJlT7DvDqpktDLxvTkL5Z5CLkX63TzDMK1LL2AK36sSqPthy1FTDNmDMry867pfjy_tktKjsI_lC40IKZwmVXEqGS2vl7c8URQVgbpXwRDKSr_WKIR7IIB-FMNaNWC3ugWYkLW-37zcqwd0uDrDQSJ9oPX0HkPKq99Imjhsot4x5i6rtLSQgSD7Q3lq1kvcEu6i4KhG4pA0yRZQmGCr4pzi7udG7eKTMYyJiq5HoFA446fdk6v0mWs9C7Cl3R_G45S_dH0M8dxR_zPQ",
			  "kty": "RSA",
			  "e": "AQAB"
			},
			{
			  "use": "sig",
			  "n": "pi22xDdK2fz5gclIbDIGghLDYiRO56eW2GUcboeVlhbAuhuT5mlEYIevkxdPOg5n6qICePZiQSxkwcYMIZyLkZhSJ2d2M6Szx2gDtnAmee6o_tWdroKu0DjqwG8pZU693oLaIjLku3IK20lTs6-2TeH-pUYMjEqiFMhn-hb7wnvH_FuPTjgz9i0rEdw_Hf3Wk6CMypaUHi31y6twrMWq1jEbdQNl50EwH-RQmQ9bs3Wm9V9t-2-_Jzg3AT0Ny4zEDU7WXgN2DevM8_FVje4IgztNy29XUkeUctHsr-431_Iu23JIy6U4Kxn36X3RlVUKEkOMpkDD3kd81JPW4Ger_w",
			  "e": "AQAB",
			  "alg": "RS256",
			  "kid": "b2620d5e7f132b52afe8875cdf3776c064249d04",
			  "kty": "RSA"
			}
		  ]
		})).unwrap()

        })
	}

	fn make_provider(server: &MockServer) -> HttpProvider {
		let url = Url::parse(&format!("{}/certs", server.uri())).unwrap();
		HttpProvider::new(url.clone(), client().clone())
	}

	/// Helper function to call the provider `expected_is_expired_values.len()` times
	/// and check:
	/// * JWKs equal `example_jwks()`
	/// * expiry status matches the corresponding entry in `expected_is_expired_values`
	async fn get_and_check_jwks(
		provider: &HttpProvider,
		expected_is_expired_values: &[bool],
	) {
		for &expected_is_expired in expected_is_expired_values {
			let jwks = provider.get().await.unwrap();
			assert_eq!(
				jwks.is_expired(),
				expected_is_expired,
				"expiry didn't match expected value"
			);
			assert_eq!(jwks.jwks(), example_jwks(), "jwks retrieved should match");
		}
	}

	#[traced_test]
	#[tokio::test]
	async fn test_no_headers() {
		// Arrange
		let server = MockServer::start().await;
		let provider = make_provider(&server);

		let response = ResponseTemplate::new(200).set_body_json(example_jwks());

		const NUM_REQUESTS: usize = 3;
		let _mock = Mock::given(matchers::method("GET"))
			.and(matchers::path("/certs"))
			.respond_with(response)
			// None of the requests should be cached.
			.expect(NUM_REQUESTS as u64)
			.mount_as_scoped(&server)
			.await;

		// Act + Assert
		get_and_check_jwks(&provider, &[true; NUM_REQUESTS]).await
	}

	#[traced_test]
	#[tokio::test]
	async fn test_cache_control_header_no_age() {
		// Arrange
		let server = MockServer::start().await;
		let provider = make_provider(&server);

		let response = ResponseTemplate::new(200)
			.set_body_json(example_jwks())
			.insert_header(CACHE_CONTROL, "max-age=60");

		const NUM_REQUESTS: usize = 3;
		let _mock = Mock::given(matchers::method("GET"))
			.and(matchers::path("/certs"))
			.respond_with(response)
			// All after first request should be cached
			.expect(1)
			.mount_as_scoped(&server)
			.await;

		// Act + Assert
		get_and_check_jwks(&provider, &[false; NUM_REQUESTS]).await
	}

	#[traced_test]
	#[tokio::test]
	async fn test_cache_control_header_age_zero() {
		// Arrange
		let server = MockServer::start().await;
		let provider = make_provider(&server);

		let response = ResponseTemplate::new(200)
			.set_body_json(example_jwks())
			.insert_header(CACHE_CONTROL, "max-age=60")
			.insert_header(AGE, "0");

		const NUM_REQUESTS: usize = 3;
		Mock::given(matchers::method("GET"))
			.and(matchers::path("/certs"))
			.respond_with(response)
			// All after first request should be cached
			.expect(1)
			.mount(&server)
			.await;

		// Act + Assert
		get_and_check_jwks(&provider, &[false; NUM_REQUESTS]).await
	}

	#[traced_test]
	#[tokio::test]
	async fn test_cache_control_header_age_greater_than_maxage() {
		// Arrange
		let server = MockServer::start().await;
		let provider = make_provider(&server);

		let response = ResponseTemplate::new(200)
			.set_body_json(example_jwks())
			.insert_header(CACHE_CONTROL, "max-age=60")
			.insert_header(AGE, "69");

		const NUM_REQUESTS: usize = 3;
		Mock::given(matchers::method("GET"))
			.and(matchers::path("/certs"))
			.respond_with(response)
			// None of the requests should be cached.
			.expect(NUM_REQUESTS as u64)
			.mount(&server)
			.await;

		// Act + Assert
		get_and_check_jwks(&provider, &[true; NUM_REQUESTS]).await
	}

	#[traced_test]
	#[tokio::test]
	async fn test_404_with_valid_payload() {
		// Arrange
		let server = MockServer::start().await;
		let provider = make_provider(&server);

		let response = ResponseTemplate::new(404).set_body_json(example_jwks());

		Mock::given(matchers::method("GET"))
			.and(matchers::path("/certs"))
			.respond_with(response)
			.expect(1)
			.mount(&server)
			.await;

		// Act + Assert
		assert!(provider.get().await.is_err());
	}
}
