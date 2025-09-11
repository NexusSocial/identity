use bytes::Bytes;
use color_eyre::{Result, eyre::Context};
use iroh_gossip::net::Gossip;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument as _, debug, info_span, instrument};

use crate::ProtectedTopic;

#[derive(Debug)]
pub(crate) struct TopicHandle {
	task: tokio::task::JoinHandle<Result<()>>,
	data_to_publish: watch::Sender<Bytes>,
}

#[bon::bon]
impl TopicHandle {
	#[builder]
	pub fn spawn(
		topic: ProtectedTopic,
		cancel: CancellationToken,
		gossip: Gossip,
	) -> Self {
		let (tx, rx) = watch::channel(Bytes::new());
		let task = tokio::task::spawn(
			main()
				.topic(topic)
				.gossip(gossip)
				.cancel(cancel)
				.rx(rx)
				.call(),
		);

		Self {
			task,
			data_to_publish: tx,
		}
	}
}

#[bon::builder]
#[instrument(skip_all, fields(topic))]
async fn main(
	cancel: CancellationToken,
	mut rx: watch::Receiver<Bytes>,
	gossip: Gossip,
	topic: ProtectedTopic,
) -> Result<()> {
	let gossip_topic = gossip
		.subscribe(topic.id(), vec![])
		.await
		.wrap_err("failed to subscribe to gossip topic")?; // empty becuase we *are* the bootstrap
	while let Ok(()) = rx.changed().await {
		//
	}
	debug!("exiting");

	Ok(())
}
