use zlambda_core::common::async_trait;
use zlambda_core::common::fs::read;
use zlambda_core::common::future::stream::{empty, StreamExt};
use zlambda_core::common::module::{Module as CommonModule, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::utility::Bytes;
use zlambda_core::server::{
    ServerBuilder, ServerId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput,
};
use wasmer_compiler_llvm::LLVM;
use wasmer::{imports, Instance, Module, Store, Value};
use rand::{RngCore, thread_rng};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct ImmediateWasmExecutor {}

#[async_trait]
impl CommonModule for ImmediateWasmExecutor {}

#[async_trait]
impl ServerModule for ImmediateWasmExecutor {
    async fn on_notification(
        &self,
        mut input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        while let Some(bytes) = input
            .notification_body_item_queue_receiver_mut()
            .next()
            .await
        {
            println!("{:?}", bytes);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

const MATRIX_SIZE: usize = 128;

////////////////////////////////////////////////////////////////////////////////////////////////////

impl ImmediateWasmExecutor {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut left: [u8; MATRIX_SIZE * MATRIX_SIZE] = [0; MATRIX_SIZE * MATRIX_SIZE];
        let mut right: [u8; MATRIX_SIZE * MATRIX_SIZE] = [0; MATRIX_SIZE * MATRIX_SIZE];
        rng.fill_bytes(&mut left);
        rng.fill_bytes(&mut right);

        let mut store = Store::new(LLVM::default());
        let module = Module::new(&store, include_bytes!("../../target/wasm32-unknown-unknown/release/zlambda_wasm_matrix.wasm")).expect("");

        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();

        {
            let memory = instance.exports.get_memory("memory").unwrap();
            memory.view(&store).write(1, &left).unwrap();
            memory.view(&store).write(1 + left.len() as u64, &right).unwrap();
        }

        let main = instance.exports.get_function("main").expect("");
        main.call(&mut store, &[Value::I32(1)]).expect("");

        let memory = instance.exports.get_memory("memory").unwrap();
        let mut result: [u8; MATRIX_SIZE * MATRIX_SIZE] = [0; MATRIX_SIZE * MATRIX_SIZE];
        memory.view(&store).read((1) as _, &mut result).unwrap();

        println!("{:?}", result);

        let memory = instance.exports.get_memory("memory").unwrap();
        let mut result: [u8; MATRIX_SIZE * MATRIX_SIZE] = [0; MATRIX_SIZE * MATRIX_SIZE];
        memory.view(&store).read((1 + left.len()) as _, &mut result).unwrap();

        println!("{:?}", result);

        let mut result: [u8; MATRIX_SIZE * MATRIX_SIZE] = [0; MATRIX_SIZE * MATRIX_SIZE];
        memory.view(&store).read((1 + left.len() + right.len()) as _, &mut result).unwrap();

        println!("{:?}", result);

        Self {}
    }
}
