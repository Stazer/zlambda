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
use wasmer::{Memory, MemoryType, imports, Instance, Module, Store, Value, ExternRef};

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

impl ImmediateWasmExecutor {
    pub fn new() -> Self {
        let mut store = Store::new(LLVM::default());
        let module = Module::new(&store, include_bytes!("../../zlambda-wasm-matrix/pkg/zlambda_wasm_matrix_bg.wasm")).expect("");

        //let memory = Memory::new(&mut store, MemoryType::new(1, None, false)).unwrap();

        let data = [4, 5, 6, 7];

        let a = ExternRef::new(&mut store, data);
        // The module doesn't import anything, so we create an empty import object.
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).expect("");

        let add_one = instance.exports.get_function("calculate").expect("");
        let result = add_one.call(&mut store, &[data.into(), Value::I32(2), Value::I32(2)]).expect("");

        println!("DATA: {:?}", result);

        //println!("hello world");

        Self {}
    }
}
