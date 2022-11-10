use crate::module::ModuleVersion;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleSpecification {
    name: String,
    description: String,
    version: ModuleVersion,
}

impl ModuleSpecification {
    pub fn new(name: String, description: String, version: ModuleVersion) -> Self {
        Self {
            name,
            description,
            version,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn version(&self) -> &ModuleVersion {
        &self.version
    }
}
