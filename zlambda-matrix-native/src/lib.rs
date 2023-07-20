use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::async_trait;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::bytes::BytesMut;
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputSource, ServerModuleNotificationEventOutput,
    ServerNotificationOrigin,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct MatrixCalculator;

#[async_trait]
impl Module for MatrixCalculator {}

#[async_trait]
impl ServerModule for MatrixCalculator {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, source, notification_body_item_queue_receiver) = input.into();

        let origin = match source {
            ServerModuleNotificationEventInputSource::Server(server) => {
                if let Some(origin) = server.origin() {
                    Some(ServerNotificationOrigin::new(
                        origin.server_id(),
                        origin.server_client_id(),
                    ))
                } else {
                    None
                }
            }
            ServerModuleNotificationEventInputSource::Client(client) => Some(
                ServerNotificationOrigin::new(server.server_id().await, client.server_client_id()),
            ),
        };

        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        //let mut data = BytesMut::with_capacity(128 * 128 * 3);

        let mut data: [u8; 128 * 128 * 3] = [0; 128 * 128 * 3];
        let mut written = 0;

        while let Some(bytes) = deserializer.next().await {
            //data[written..].copy_from_slice(&bytes);
            written += bytes.len();
        }

        println!("hello {}", written);

        /*let left = unsafe { from_raw_parts(data, MATRIX_SIZE * MATRIX_SIZE) };
        let right = unsafe {
            from_raw_parts(
                data.add(MATRIX_SIZE * MATRIX_SIZE),
                MATRIX_SIZE * MATRIX_SIZE,
            )
        };
        let result = unsafe {
            from_raw_parts_mut(
                data.add(MATRIX_SIZE * MATRIX_SIZE * 2),
                MATRIX_SIZE * MATRIX_SIZE,
            )
        };

        for i in 0..MATRIX_SIZE {
            for j in 0..MATRIX_SIZE {
                let mut value = 0;

                for k in 0..MATRIX_SIZE {
                    let (left_value, right_value) = match (
                        left.get(i * MATRIX_SIZE + k),
                        right.get(k * MATRIX_SIZE + j),
                    ) {
                        (Some(left_value), Some(right_value)) => (left_value, right_value),
                        (_, _) => return,
                    };

                    value += left_value * right_value;
                }

                if let Some(old_value) = result.get_mut(i * MATRIX_SIZE + j) {
                    *old_value = value;
                }
            }
        }*/
    }
}
