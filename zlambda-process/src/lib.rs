use zlambda_common::{Module};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleImpl {

}

impl Module for ModuleImpl {

}

#[no_mangle]
pub extern "C" fn modules() -> Vec<*const dyn Module> {
    let module = Box::new(ModuleImpl {});
    vec!(Box::into_raw(module))
}
