use std::future::Future;

pub use pkarr::Client;

use crate::{doc::TryFromSignedPacketErr, DidPkarr, DidPkarrDocument};

#[derive(Debug, thiserror::Error)]
pub enum ResolveErr {
	#[error("could not resolve with PKARR")]
	NotFound,
	#[error("failed to convert from pkarr into DID Document")]
	Invalid(#[from] TryFromSignedPacketErr),
}

#[cfg(any(feature = "dht", feature = "http"))]
pub trait PkarrClientExt {
	fn resolve(
		&self,
		did: &DidPkarr,
	) -> impl Future<Output = Result<DidPkarrDocument, ResolveErr>> + Send;

	fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> impl Future<Output = Result<DidPkarrDocument, ResolveErr>> + Send;
}

#[cfg(any(feature = "dht", feature = "http"))]
pub trait PkarrClientBlockingExt {
	fn resolve(&self, did: &DidPkarr) -> Result<DidPkarrDocument, ResolveErr>;

	fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> Result<DidPkarrDocument, ResolveErr>;
}

#[cfg(any(feature = "dht", feature = "http"))]
impl PkarrClientExt for pkarr::Client {
	async fn resolve(&self, did: &DidPkarr) -> Result<DidPkarrDocument, ResolveErr> {
		{
			let public_key = pkarr::PublicKey::try_from(did.as_pubkey()).unwrap();
			let Some(packet) = self.resolve(&public_key).await else {
				return Err(ResolveErr::NotFound);
			};

			DidPkarrDocument::try_from(packet).map_err(ResolveErr::from)
		}
	}

	async fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> Result<DidPkarrDocument, ResolveErr> {
		let public_key = pkarr::PublicKey::try_from(did.as_pubkey()).unwrap();
		let Some(packet) = self.resolve_most_recent(&public_key).await else {
			return Err(ResolveErr::NotFound);
		};

		DidPkarrDocument::try_from(packet).map_err(ResolveErr::from)
	}
}

#[cfg(any(feature = "dht", feature = "http"))]
impl PkarrClientBlockingExt for pkarr::ClientBlocking {
	fn resolve(&self, did: &DidPkarr) -> Result<DidPkarrDocument, ResolveErr> {
		{
			let public_key = pkarr::PublicKey::try_from(did.as_pubkey()).unwrap();
			let Some(packet) = self.resolve(&public_key) else {
				return Err(ResolveErr::NotFound);
			};

			DidPkarrDocument::try_from(packet).map_err(ResolveErr::from)
		}
	}

	fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> Result<DidPkarrDocument, ResolveErr> {
		let public_key = pkarr::PublicKey::try_from(did.as_pubkey()).unwrap();
		let Some(packet) = self.resolve_most_recent(&public_key) else {
			return Err(ResolveErr::NotFound);
		};

		DidPkarrDocument::try_from(packet).map_err(ResolveErr::from)
	}
}
