use std::path::PathBuf;

use super::super::core;

// use crate::modules::script::engine::FromLuaTable;

/// `KindConfig` is a struct that holds the configuration for the kind cluster.
/// - `version`: The version of the kind cluster to use.
/// - `config`: The optional path to the kind config file.
/// - `context`: The name of the kind cluster.
pub struct KindConfig {
    pub version: String,
    pub config: Option<PathBuf>,
    pub context: String,
}

// impl FromLuaTable for KindConfig {
//     fn from_lua(lua_table: Table<'_>, _ctx: Context<'_>) -> Result<Self, Err> {
//         let version: Option<String> = lua_table.get("version").ok();
//         let config: Option<String> = lua_table.get("config").ok();
//         let context: Option<String> = lua_table.get("context").ok();
//
//         let default = KindConfig::default();
//
//         // Set default values if they are missing.
//         let version = version.unwrap_or(default.version.to_string());
//
//         Ok(Self { version, config, context })
//     }
// }

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
