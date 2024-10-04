//! Structs representing deserialized config file
//!
//! See [`Config`].

use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_CONTENTS: &str = include_str!("../default_config.toml");

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum DatabaseConfig {
	Sqlite { db_file: PathBuf },
}

impl Default for DatabaseConfig {
	fn default() -> Self {
		Self::Sqlite {
			db_file: PathBuf::from(".").join("identities.db"),
		}
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct CacheSettings {
	/// If `None`, relies on `XDG_CACHE_HOME` instead.
	pub dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
	/// If `0`, uses a random available port.
	pub port: u16,
}

impl Default for HttpConfig {
	fn default() -> Self {
		Self { port: 80 }
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpsConfig {
	/// If `0`, uses a random available port.
	pub port: u16,
	#[serde(default)]
	pub tls: TlsConfig,
}

impl Default for HttpsConfig {
	fn default() -> Self {
		Self {
			port: 443,
			tls: TlsConfig::default(),
		}
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ThirdPartySettings {
	#[serde(default = "default_some")]
	pub google: Option<GoogleSettings>,
}

impl Default for ThirdPartySettings {
	fn default() -> Self {
		Self {
			google: Some(GoogleSettings::default()),
		}
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GoogleSettings {
	/// The Google API OAuth2 Client ID.
	/// See https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid
	#[serde(default)]
	pub oauth2_client_id: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum TlsConfig {
	/// LetsEncrypt's certificate authoriy and the TLS-ALPN-01 challenge type to get a
	/// valid signed certificate.
	/// Read more at https://letsencrypt.org/docs/challenge-types/#tls-alpn-01
	Acme {
		domains: Vec<String>,
	},
	/// Creates a self-signed certificate
	SelfSigned {
		domains: Vec<String>,
	},
	File {
		path: PathBuf,
	},
}

impl Default for TlsConfig {
	fn default() -> Self {
		Self::Acme {
			domains: Vec::new(),
		}
	}
}

/// Helper function to construct `Some(T::default())`.
fn default_some<T: Default>() -> Option<T> {
	Some(T::default())
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("error in toml file: {0}")]
	Toml(#[from] toml::de::Error),
}

/// The contents of the config file. Contains all settings customizeable during
/// deployment.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
	#[serde(default)]
	pub database: DatabaseConfig,
	#[serde(default = "default_some")]
	pub http: Option<HttpConfig>,
	#[serde(default = "default_some")]
	pub https: Option<HttpsConfig>,
	#[serde(default)]
	pub cache: CacheSettings,
	#[serde(default)]
	pub third_party: ThirdPartySettings,
}

impl FromStr for Config {
	type Err = ConfigError;

	fn from_str(str: &str) -> Result<Self, Self::Err> {
		let config: Self = toml::from_str(str)?;
		Ok(config)
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			database: DatabaseConfig::default(),
			http: Some(HttpConfig::default()),
			https: Some(HttpsConfig::default()),
			cache: CacheSettings::default(),
			third_party: ThirdPartySettings::default(),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	/// We could have used Config::default, but I wanted to explicitly write it all out
	/// in case something is messed up.
	fn default_config() -> Config {
		Config {
			database: DatabaseConfig::Sqlite {
				db_file: PathBuf::from("./identities.db"),
			},
			http: Some(HttpConfig { port: 80 }),
			https: Some(HttpsConfig {
				port: 443,
				tls: TlsConfig::Acme {
					domains: Vec::new(),
				},
			}),
			cache: CacheSettings { dir: None },
			third_party: ThirdPartySettings {
				google: Some(GoogleSettings {
					oauth2_client_id: String::new(),
				}),
			},
		}
	}

	#[test]
	fn test_empty_config_file_deserializes_to_default() {
		let config = Config::from_str("").unwrap();
		assert_eq!(config, default_config());
		assert_eq!(config, Config::default());
	}

	#[test]
	fn test_default_config_deserializes_correctly() {
		let deserialized: Config = toml::from_str(DEFAULT_CONFIG_CONTENTS)
			.expect("default config file should always deserialize");
		assert_eq!(deserialized, Config::default());
	}
}
