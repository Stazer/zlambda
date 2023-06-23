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
use wasmer::{imports, Instance, Module, Store, Value};

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
    let module_wat = r#"
    (module
      (type $t0 (func (param i32) (result i32)))
      (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.add))
    "#;

        let module_wat = include_bytes!("../../zlambda-wasm-matrix/pkg/zlambda_wasm_matrix_bg.wasm");

        let mut store = Store::default();
        let module = Module::new(&store, &module_wat).expect("");
        // The module doesn't import anything, so we create an empty import object.
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).expect("");

        let add_one = instance.exports.get_function("greet").expect("");
        let result = add_one.call(&mut store, &[]).expect("");
        println!("DATA: {:?}",result);

        println!("hello world");

        Self {}
    }
}
