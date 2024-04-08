
use async_trait::async_trait;
use futures_util::SinkExt;
use network::{Acknowledgement};
use tokio::sync::mpsc::UnboundedSender;
use types::WrapperMsg;

#[derive(Debug, Clone)]
pub struct Handler {
    consensus_tx: UnboundedSender<WrapperMsg>,
}

impl Handler {
    pub fn new(consensus_tx: UnboundedSender<WrapperMsg>) -> Self {
        Self { consensus_tx }
    }
}

#[async_trait]
impl network::Handler<Acknowledgement, WrapperMsg>
    for Handler
{
    async fn dispatch(
        &self,
        msg: WrapperMsg,
        writer: &mut network::Writer<Acknowledgement>,
    ) {
        // Forward the message
        self.consensus_tx
            .send(msg)
            .expect("Failed to send message to the consensus channel");

        // Acknowledge
        writer
            .send(Acknowledgement::Pong)
            .await
            .expect("Failed to send an acknowledgement");
    }
}
