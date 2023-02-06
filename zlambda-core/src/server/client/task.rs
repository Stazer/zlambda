use crate::common::runtime::spawn;
use crate::server::client::ServerClientId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub(crate) struct ServerClientTask {
    client_id: ServerClientId, /*server_queue_sender: MessageQueueSender<ServerMessage>,
                               general_socket: Option<(
                                   MessageSocketSender<GeneralMessage>,
                                   MessageSocketReceiver<GeneralMessage>,
                               )>,
                               sender: MessageQueueSender<ServerNodeMessage>,
                               receiver: MessageQueueReceiver<ServerNodeMessage>,*/
}

impl ServerClientTask {
    /*pub fn new(

    )*/

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await
        }
    }

    async fn select(&mut self) {}
}
