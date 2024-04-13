use async_trait::async_trait;
use futures_util::SinkExt;
use network::{Acknowledgement};
use tokio::sync::mpsc::UnboundedSender;
use types::SyncMsg;

#[derive(Debug, Clone)]
pub struct SyncHandler {
    consensus_tx: UnboundedSender<SyncMsg>,
}

impl SyncHandler {
    pub fn new(consensus_tx: UnboundedSender<SyncMsg>) -> Self {
        Self { consensus_tx }
    }
}

#[async_trait]
impl network::Handler<Acknowledgement, SyncMsg>
for SyncHandler
{
    async fn dispatch(
        &self,
        msg: SyncMsg,
        writer: &mut network::Writer<Acknowledgement>,
    ) {
        // Forward the message to internal channel
        self.consensus_tx
            .send(msg)
            .expect("Failed to send message to the consensus channel");

        // Acknowledge the received sync message
        writer
            .send(Acknowledgement::Pong)
            .await
            .expect("Failed to send an acknowledgement");
    }
}
