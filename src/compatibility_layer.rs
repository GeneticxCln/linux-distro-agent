// Universal Package Compatibility Layer - Initial Implementation

pub mod compatibility_layer {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PackageMapping {
        pub package_name: String,      // Name of the package
        pub compatible_managers: Vec<PackageManager>, // Compatible package managers
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PackageManager {
        pub name: String,             // Name of the package manager
        pub install_command: String,  // Install command template
        pub search_command: String,   // Search command template
    }

    pub struct CompatibilityLayer {
        pub mappings: HashMap<String, PackageMapping>, // Keyed by package name
    }

    impl CompatibilityLayer {
        pub fn new() -> Self {
            Self {
                mappings: HashMap::new(),
            }
        }

        pub fn add_mapping(&mut self, mapping: PackageMapping) {
            self.mappings.insert(mapping.package_name.clone(), mapping);
        }

        pub fn get_install_command(&self, package_name: &str, manager_name: &str) -> Option<String> {
            self.mappings.get(package_name).and_then(|mapping| {
                mapping.compatible_managers.iter()
                    .find(|manager| manager.name == manager_name)
                    .map(|manager| manager.install_command.clone())
            })
        }
    }
}

