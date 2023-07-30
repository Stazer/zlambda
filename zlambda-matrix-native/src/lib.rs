use byteorder::{ByteOrder, LittleEndian};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::time::Instant;
use zlambda_core::common::async_trait;
use zlambda_core::common::bytes::BytesMut;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{
    notification_body_item_queue, NotificationBodyItemStreamExt,
};
use zlambda_core::common::runtime::spawn;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventInputSource,
    ServerModuleNotificationEventOutput,
};
use zlambda_matrix::{MATRIX_DIMENSION_SIZE, MATRIX_ELEMENT_COUNT};

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
        let program_begin = Instant::now();

        let (server, source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let mut data = BytesMut::zeroed(MATRIX_ELEMENT_COUNT * 3 + size_of::<u128>() * 2);
        let mut written = 0;

        while let Some(mut bytes) = deserializer.next().await {
            if written == 0 {
                // Split body item type header
                let _ = bytes.split_to(14);
            }

            unsafe {
                copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr().add(written), bytes.len());
            }

            written += bytes.len();
        }

        let left = unsafe {
            from_raw_parts(
                data.as_mut_ptr(),
                MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE,
            )
        };
        let right = unsafe {
            from_raw_parts(
                data.as_mut_ptr()
                    .add(MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE),
                MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE,
            )
        };
        let result = unsafe {
            from_raw_parts_mut(
                data.as_mut_ptr()
                    .add(MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE * 2),
                MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE,
            )
        };
        let times = unsafe {
            from_raw_parts_mut(
                data.as_mut_ptr()
                    .add(MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE * 3),
                size_of::<u128>() * 2,
            )
        };

        let calculation_begin = Instant::now();

        for i in 0..MATRIX_DIMENSION_SIZE {
            for j in 0..MATRIX_DIMENSION_SIZE {
                let mut value: usize = 0;

                for k in 0..MATRIX_DIMENSION_SIZE {
                    let (left_value, right_value) = match (
                        left.get(i * MATRIX_DIMENSION_SIZE + k),
                        right.get(k * MATRIX_DIMENSION_SIZE + j),
                    ) {
                        (Some(left_value), Some(right_value)) => (left_value, right_value),
                        (_, _) => return,
                    };

                    value += (*left_value as usize) * (*right_value as usize);
                }

                if let Some(old_value) = result.get_mut(i * MATRIX_DIMENSION_SIZE + j) {
                    *old_value = value as u8;
                }
            }
        }

        let calculation_end = calculation_begin.elapsed().as_nanos();

        let (sender, receiver) = notification_body_item_queue();

        spawn(async move {
            match source {
                ServerModuleNotificationEventInputSource::Server(server_source) => {
                    if let Some(origin) = server_source.origin() {
                        if let Some(origin_server) = server.servers().get(origin.server_id()).await
                        {
                            origin_server
                                .clients()
                                .get(origin.server_client_id())
                                .notify(0.into(), receiver)
                                .await;
                        }
                    }
                }
                ServerModuleNotificationEventInputSource::Client(client_source) => {
                    if let Some(client) = server
                        .local_clients()
                        .get(client_source.server_client_id())
                        .await
                    {
                        client.notify(0.into(), receiver).await;
                    }
                }
            }
        });

        LittleEndian::write_u128(times, calculation_end);
        LittleEndian::write_u128(times, program_begin.elapsed().as_nanos());

        sender.do_send(data.freeze()).await;
    }
}
