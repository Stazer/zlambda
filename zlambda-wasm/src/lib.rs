use wasmer::{imports, Instance, Module, Store, Value};
use wasmer_compiler_llvm::LLVM;
use zlambda_core::common::async_trait;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::Module as CommonModule;
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

const MATRIX_DIMENSION_SIZE: usize = 128;
const MATRIX_ELEMENT_COUNT: usize = MATRIX_DIMENSION_SIZE * MATRIX_DIMENSION_SIZE;

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
        let (_server, _source, notification_body_item_queue_receiver) = input.into();
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

        let mut csum: usize = 0;

        while let Some(mut bytes) = deserializer.next().await {
            if written == 0 {
                // Split body item type header
                let _ = bytes.split_to(14);
            }

            for byte in &bytes {
                csum += *byte as usize;
            }

            let memory = instance.exports.get_memory("memory").unwrap();
            memory.view(&store).write(1 + written, &bytes).unwrap();

            written += bytes.len() as u64;
        }

        let main = instance.exports.get_function("main").expect("");
        main.call(&mut store, &[Value::I32(1)]).expect("");

        let memory = instance.exports.get_memory("memory").unwrap();
        let mut result: [u8; MATRIX_ELEMENT_COUNT] = [0; MATRIX_ELEMENT_COUNT];
        memory
            .view(&store)
            .read((1 + 2 * MATRIX_ELEMENT_COUNT) as _, &mut result)
            .unwrap();

        println!("hello {} {}", csum, written);
    }
}
