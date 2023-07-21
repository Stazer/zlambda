use wasmer::{imports, Instance, Module, Store, Value};
use wasmer_compiler_llvm::LLVM;
use zlambda_core::common::async_trait;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::runtime::spawn;
use zlambda_core::common::module::Module as CommonModule;
use zlambda_core::common::bytes::BytesMut;
use zlambda_core::common::notification::{
    notification_body_item_queue, NotificationBodyItemStreamExt,
};
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerModuleNotificationEventInputSource,
};
use zlambda_matrix::{
    MATRIX_ELEMENT_COUNT,
};

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
        let (server, source, notification_body_item_queue_receiver) = input.into();
        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let mut store = Store::new(LLVM::default());
        let module = Module::new(
            &store,
            include_bytes!("../../target/wasm32-unknown-unknown/release/zlambda_wasm_matrix.wasm"),
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

            let memory = instance.exports.get_memory("memory").unwrap();
            memory.view(&store).write(1 + written, &bytes).unwrap();

            written += bytes.len() as u64;
        }

        let main = instance.exports.get_function("main").expect("");
        main.call(&mut store, &[Value::I32(1)]).expect("");

        let memory = instance.exports.get_memory("memory").unwrap();
        let mut result = BytesMut::zeroed(MATRIX_ELEMENT_COUNT);
        memory
            .view(&store)
            .read((1 + 2 * MATRIX_ELEMENT_COUNT) as _, &mut result)
            .unwrap();

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

        sender.do_send(result.freeze()).await;
    }
}
