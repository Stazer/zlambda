use byteorder::{ByteOrder, LittleEndian};
use std::mem::size_of;
use std::time::Instant;
use wasmer::{imports, Instance, Module, Store, Value};
use wasmer_compiler_llvm::LLVM;
use zlambda_core::common::async_trait;
use zlambda_core::common::bytes::BytesMut;
use std::slice::{from_raw_parts_mut};
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::Module as CommonModule;
use zlambda_core::common::notification::{
    notification_body_item_queue, NotificationBodyItemStreamExt,
};
use zlambda_core::common::runtime::spawn;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventInputSource,
    ServerModuleNotificationEventOutput,
};
use zlambda_matrix::{MATRIX_SIZE};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct ImmediateWasmExecutor {}

#[async_trait]
impl CommonModule for ImmediateWasmExecutor {}

#[async_trait]
impl ServerModule for ImmediateWasmExecutor {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let program_begin = Instant::now();

        let (server, source, notification_body_item_queue_receiver) = input.into();
        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let mut store = Store::new(LLVM::default());
        let module = Module::new(
            &store,
            include_bytes!(
                "../../target/wasm32-unknown-unknown/release/zlambda_matrix_wasm_payload.wasm"
            ),
        )
        .expect("");

        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();

        let mut written = 0;

        while let Some(mut bytes) = deserializer.next().await {
            if written == 0 {
                // Split body item type header
                let _ = bytes.split_to(14);
            }

            let memory = instance.exports.get_memory("memory").expect("");
            memory.view(&store).write(1 + written, &bytes).expect("-");

            written += bytes.len() as u64;
        }

        let main = instance.exports.get_function("main").expect("");
        let calculation_begin = Instant::now();
        main.call(&mut store, &[Value::I32(1)]).expect("");
        let calculation_end = calculation_begin.elapsed().as_nanos();

        let memory = instance.exports.get_memory("memory").expect("");
        let mut result = BytesMut::zeroed(MATRIX_SIZE + size_of::<u128>() * 2);
        memory
            .view(&store)
            .read((1 + 2 * MATRIX_SIZE) as _, &mut result[0..MATRIX_SIZE])
            .expect("");

        let times = unsafe {
            from_raw_parts_mut(
                result.as_mut_ptr()
                    .add(MATRIX_SIZE),
                size_of::<u128>() * 2,
            )
        };

        let times2 = unsafe {
            from_raw_parts_mut(
                result.as_mut_ptr()
                    .add(MATRIX_SIZE + size_of::<u128>()),
                size_of::<u128>(),
            )
        };

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

        let program_end = program_begin.elapsed().as_nanos();

        LittleEndian::write_u128(times, calculation_end);
        LittleEndian::write_u128(times2, program_end);

        sender.do_send(result.freeze()).await;
    }
}
