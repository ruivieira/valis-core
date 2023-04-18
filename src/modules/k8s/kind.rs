use std::path::PathBuf;

use super::modules::core as core;

/// `KindConfig` is a struct that holds the configuration for the kind cluster.
/// - `version`: The version of the kind cluster to use.
/// - `config`: The optional path to the kind config file.
/// - `context`: The name of the kind cluster.
pub struct KindConfig {
    pub version: String,
    pub config: Option<PathBuf>,
    pub context: String,
}

/// `default` is a function that returns a default [`KindConfig`] struct.
/// The default [`KindConfig`] struct has the following values:
/// version: Default is "1.22.15"
/// config: None
/// context: "kind"
impl Default for KindConfig {
    fn default() -> Self {
        KindConfig {
            version: "1.22.15".to_owned(),
            config: None,
            context: "kind".to_owned(),
        }
    }
}

/// `start` is a function that starts a kind cluster by providing the [`KindConfig`] struct.
pub fn start(config: KindConfig) {
    let mut command = "kind create cluster".to_owned();
    command.push_str(&format!(" --image=kindest/node:v{}", config.version));

    if config.config.is_some() {
        command.push_str(&format!("--config {:?}", config.config))
    }

    core::run(&command);
}