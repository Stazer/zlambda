use std::error::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use zlambda_common::channel::{DoReceive, DoSend};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct CandidatePingMessage {
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum CandidateMessage {
    Ping(CandidatePingMessage),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CandidateHandle {
    sender: mpsc::Sender<CandidateMessage>,
}

impl CandidateHandle {
    fn new(sender: mpsc::Sender<CandidateMessage>) -> Self {
        Self { sender }
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(CandidateMessage::Ping(CandidatePingMessage { sender }))
            .await;

        receiver.do_receive().await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CandidateBuilder {
    sender: mpsc::Sender<CandidateMessage>,
    receiver: mpsc::Receiver<CandidateMessage>,
}

impl CandidateBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> CandidateHandle {
        CandidateHandle::new(self.sender.clone())
    }

    pub async fn task(self) -> Result<CandidateTask, Box<dyn Error>> {
        CandidateTask::new(self.receiver).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CandidateTask {
    receiver: mpsc::Receiver<CandidateMessage>,
}

impl CandidateTask {
    async fn new(receiver: mpsc::Receiver<CandidateMessage>) -> Result<Self, Box<dyn Error>> {
        Ok(Self { receiver })
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await;
        }
    }

    async fn select(&mut self) {
        select!(
            result = self.receiver.recv() => {
                if let Some(message) = result {
                    self.on_candidate_message(message).await;
                }
            }
        )
    }

    async fn on_candidate_message(&mut self, message: CandidateMessage) {
        match message {
            CandidateMessage::Ping(message) => self.on_candidate_ping_message(message).await,
        }
    }

    async fn on_candidate_ping_message(&mut self, message: CandidatePingMessage) {
        message.sender.do_send(()).await
    }
}
