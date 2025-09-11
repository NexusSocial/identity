use std::{ops::Deref, sync::Arc};

use bytes::Bytes;
use dashmap::DashMap;
use did_common::did::Did;
use iroh::{Endpoint, SecretKey};
use iroh_gossip::{api::GossipSender, net::Gossip, proto::TopicId};
use sha2::{Digest, Sha256};

use crate::topic::TopicHandle;

mod topic;

const HASH_CTX: &str = "did-pub-sub/v0";

#[derive(Debug)]
pub struct ClientInner {
	endpoint: iroh::Endpoint,
	gossip: Gossip,
	topics: DashMap<ProtectedTopic, TopicHandle>,
}

impl ClientInner {
	pub fn new(endpoint: &Endpoint) -> Self {
		let endpoint = endpoint.clone();
		let gossip = Gossip::builder().spawn(endpoint.clone());
		let topics = DashMap::new();

		Self {
			endpoint,
			gossip,
			topics,
		}
	}
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
