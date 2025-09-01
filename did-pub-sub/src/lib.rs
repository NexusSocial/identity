use std::{ops::Deref, sync::Arc};

use bytes::Bytes;
use dashmap::DashMap;
use did_common::did::Did;
use iroh::{Endpoint, SecretKey};
use iroh_gossip::{api::GossipSender, net::Gossip, proto::TopicId};
use sha2::{Digest, Sha256};

const HASH_CTX: &str = "did-pub-sub/v0";

#[derive(Debug, Clone)]
pub struct Client {
	endpoint: iroh::Endpoint,
	gossip: Gossip,
	write_protected: Arc<DashMap<ProtectedTopic, Arc<ProtectedTopicData>>>,
}

impl Client {
	pub fn new(endpoint: &Endpoint) -> Self {
		let endpoint = endpoint.clone();
		let gossip = Gossip::builder().spawn(endpoint.clone());
		let write_protected = Arc::new(DashMap::new());

		Client {
			endpoint,
			gossip,
			write_protected,
		}
	}

	pub fn write_protected(
		&self,
		topic: ProtectedTopic,
		secret: SecretKey,
		message: Bytes,
	) {
		let topic_data = if let Some(sender) = self.write_protected.get(&topic) {
			sender.clone()
		} else {
			todo!()
		};
	}

	pub fn read_protected(&self, topic: ProtectedTopic) -> Option<Bytes> {
		todo!()
	}
}

#[derive(Debug)]
struct ProtectedTopicData {
	msg: Bytes,
	iroh_topic: GossipSender,
}

/// A topic that can only be published to by a particular DID.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProtectedTopic {
	topic_name: String,
	publisher: Did,
	id: TopicId,
}

#[bon::bon]
impl ProtectedTopic {
	#[builder]
	pub fn new(topic_name: String, publisher: Did) -> ProtectedTopic {
		let mut hasher = Sha256::new_with_prefix(HASH_CTX);
		hasher.update(&topic_name);
		hasher.update(publisher.as_str());
		let hash = hasher.finalize();
		let id = TopicId::from_bytes(hash.into());

		ProtectedTopic {
			topic_name,
			publisher,
			id,
		}
	}

	fn id(&self) -> TopicId {
		self.id
	}
}
