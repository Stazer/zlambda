use zlambda_common::{Module};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleImpl {

}

impl Module for ModuleImpl {

}

#[no_mangle]
pub extern "C" fn modules() -> Vec<Box<dyn Module>> {
    vec!(
        Box::new(ModuleImpl{})
    )
}
