use std::{future::Future, time::SystemTime};

pub use pkarr::Client;
use pkarr::Timestamp;

use crate::{
	doc::{ToPkarrErr, TryFromSignedPacketErr},
	DidPkarr, DidPkarrDocument,
};

#[derive(Debug, thiserror::Error)]
pub enum ResolveErr {
	#[error("could not resolve with PKARR")]
	NotFound,
	#[error("failed to convert from pkarr into DID Document")]
	Invalid(#[from] TryFromSignedPacketErr),
}

#[derive(Debug, thiserror::Error)]
pub enum PublishErr {
	#[error("failed to convert from DID Document to pkarr")]
	ToPkarr(#[from] ToPkarrErr),
	#[error("failed to publish with pkarr client")]
	IoErr(#[from] pkarr::errors::PublishError),
}

#[cfg(any(feature = "dht", feature = "http"))]
pub trait PkarrClientExt {
	/// Like [`pkarr::Client::resolve`] but for DIDs.
	fn resolve(
		&self,
		did: &DidPkarr,
	) -> impl Future<Output = Result<DidPkarrDocument, ResolveErr>> + Send;

	/// Like [`pkarr::Client::resolve_most_recent`] but for DIDs.
	fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> impl Future<Output = Result<DidPkarrDocument, ResolveErr>> + Send;

	/// Like [`pkarr::Client::publish`] but for DIDs.
	fn publish(
		&self,
		doc: &DidPkarrDocument,
		timestamp: Option<Timestamp>,
		signing_key: &ed25519_dalek::SigningKey,
	) -> impl Future<Output = Result<(), PublishErr>> + Send;
}

#[cfg(any(feature = "dht", feature = "http"))]
pub trait PkarrClientBlockingExt {
	/// Like [`pkarr::Client::resolve`] but for DIDs.
	fn resolve(&self, did: &DidPkarr) -> Result<DidPkarrDocument, ResolveErr>;

	/// Like [`pkarr::Client::resolve_most_recent`] but for DIDs.
	fn resolve_most_recent(
		&self,
		did: &DidPkarr,
	) -> Result<DidPkarrDocument, ResolveErr>;

	/// Like [`pkarr::Client::publish`] but for DIDs.
	fn publish(
		&self,
		doc: &DidPkarrDocument,
		timestamp: Option<Timestamp>,
		signing_key: &ed25519_dalek::SigningKey,
	) -> Result<(), PublishErr>;
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

	async fn publish(
		&self,
		doc: &DidPkarrDocument,
		timestamp: Option<Timestamp>,
		signing_key: &ed25519_dalek::SigningKey,
	) -> Result<(), PublishErr> {
		let timestamp = if let Some(timestamp) = timestamp {
			timestamp
		} else {
			SystemTime::now().into()
		};
		let signed_packet = doc.to_pkarr_packet(signing_key, timestamp)?;

		self.publish(&signed_packet, Some(timestamp))
			.await
			.map_err(PublishErr::from)
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

	fn publish(
		&self,
		doc: &DidPkarrDocument,
		timestamp: Option<Timestamp>,
		signing_key: &ed25519_dalek::SigningKey,
	) -> Result<(), PublishErr> {
		let timestamp = if let Some(timestamp) = timestamp {
			timestamp
		} else {
			SystemTime::now().into()
		};
		let signed_packet = doc.to_pkarr_packet(signing_key, timestamp)?;

		self.publish(&signed_packet, Some(timestamp))
			.map_err(PublishErr::from)
	}
}
