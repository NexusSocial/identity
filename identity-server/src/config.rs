//! Structs representing deserialized config file
//!
//! See [`Config`].

use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_CONTENTS: &str = include_str!("../default-config.toml");
const CACHE_DIR_SUFFIX: &str = "nexus_identity_server";

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
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
	dir: Option<PathBuf>,
}

impl CacheSettings {
	pub fn dir(&self) -> PathBuf {
		if let Some(ref dir) = self.dir {
			dir.to_owned()
		} else {
			std::env::var("XDG_CACHE_HOME")
				.map(PathBuf::from)
				.or_else(|_| std::env::var("HOME").map(PathBuf::from))
				.unwrap_or_else(|_| std::env::current_dir().unwrap())
				.join(CACHE_DIR_SUFFIX)
		}
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
	/// If `0`, uses a random available port.
	#[serde(default = "HttpConfig::default_port")]
	pub port: u16,
	#[serde(default)]
	pub tls: TlsConfig,
}

impl HttpConfig {
	fn validate(&self) -> Result<(), ValidationError> {
		if let TlsConfig::Acme { ref domains, .. } = self.tls {
			if domains.is_empty() {
				return Err(ValidationError::UnspecifiedDomain);
			}
		}
		Ok(())
	}
}

impl Default for HttpConfig {
	fn default() -> Self {
		Self {
			port: Self::default_port(),
			tls: TlsConfig::default(),
		}
	}
}

impl HttpConfig {
	const fn default_port() -> u16 {
		8443
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
	/// See <https://developers.google.com/identity/gsi/web/guides/get-google-api-clientid>
	#[serde(default)]
	pub oauth2_client_id: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "snake_case")]
pub enum TlsConfig {
	/// Don't use any tls. Revert to HTTP only.
	Disable,
	/// LetsEncrypt's certificate authoriy and the TLS-ALPN-01 challenge type to get a
	/// valid signed certificate.
	/// Read more at <https://letsencrypt.org/docs/challenge-types/#tls-alpn-01>
	Acme {
		/// Whether we use the staging or prod LetsEncrypt directory.
		is_prod: bool,
		domains: Vec<String>,
		email: String,
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
			email: String::new(),
			is_prod: true,
		}
	}
}

/// Helper function to construct `Some(T::default())`.
fn default_some<T: Default>() -> Option<T> {
	Some(T::default())
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ConfigError {
	#[error("error deserializing toml file: {0}")]
	Toml(#[from] toml::de::Error),
	#[error("config file was invalid: {0}")]
	FailedValidation(#[from] ValidationError),
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ValidationError {
	#[error("when using ACME tls, you *must* specify at least one domain")]
	UnspecifiedDomain,
}

/// The contents of the config file. Contains all settings customizeable during
/// deployment.
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
	#[serde(default)]
	pub database: DatabaseConfig,
	#[serde(default)]
	pub http: HttpConfig,
	#[serde(default)]
	pub cache: CacheSettings,
	#[serde(default)]
	pub third_party: ThirdPartySettings,
}

impl Config {
	/// Validates the deserialized config
	pub fn validate(&self) -> Result<(), ValidationError> {
		self.http.validate()?;
		Ok(())
	}
}

impl FromStr for Config {
	type Err = ConfigError;

	fn from_str(str: &str) -> Result<Self, Self::Err> {
		let config: Self = toml::from_str(str)?;
		Ok(config)
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
			http: HttpConfig {
				port: 8443,
				tls: TlsConfig::Acme {
					email: String::new(),
					domains: Vec::new(),
					is_prod: true,
				},
			},
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
		let config = Config::from_str("").expect("config file should deserialize");
		assert_eq!(config, default_config());
		assert_eq!(config, Config::default());
	}

	#[test]
	fn test_default_config_deserializes_correctly_but_fails_validation() {
		let deserialized: Config = toml::from_str(DEFAULT_CONFIG_CONTENTS)
			.expect("default config file should always deserialize");
		assert_eq!(deserialized, Config::default());
		assert_eq!(
			deserialized.validate(),
			Err(ValidationError::UnspecifiedDomain)
		)
	}

	#[test]
	fn test_disabling_tls_keeps_all_other_defaults() {
		let config = Config::from_str(r#"http.tls.type = "disable""#)
			.expect("config file should deserialize");
		let expected = Config {
			http: HttpConfig {
				tls: TlsConfig::Disable,
				..HttpConfig::default()
			},
			..Config::default()
		};
		assert_eq!(config, expected);
	}

	#[test]
	fn test_database_config_with_custom_sqlite_path() {
		const CONTENTS: &str = r#"
            [database]
            type = "sqlite"
            db_file = "../../foobar.db"
        "#;
		let config =
			Config::from_str(CONTENTS).expect("config file should deserialize");
		assert_eq!(
			config,
			Config {
				database: DatabaseConfig::Sqlite {
					db_file: PathBuf::from("../../foobar.db")
				},
				..Config::default()
			}
		);
	}
}
