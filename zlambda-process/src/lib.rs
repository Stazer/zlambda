use zlambda_common::{async_trait, DispatchEvent, Module, ModuleSpecification, ModuleVersion};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleImplementation {
    version: ModuleVersion,
}

#[async_trait]
impl Module for ModuleImplementation {
    fn specification(&self) -> ModuleSpecification {
        ModuleSpecification::new(
            String::from("process"),
            String::from("A module for running local processes"),
            self.version.clone(),
        )
    }

    async fn on_dispatch(&self, event: DispatchEvent) {}
}

#[no_mangle]
pub extern "C" fn modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(ModuleImplementation {
            version: (1, 0, 0).into(),
        }),
        Box::new(ModuleImplementation {
            version: (2, 0, 0).into(),
        }),
    ]
}
