use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use rlua::{Context, Error, ExternalError, Lua};
use rlua::prelude::LuaError;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::modules::projects::git::core;

pub fn _get_git_project_root_path(ctx: &Context) {
    let f = ctx
        .create_function(|_, path: String| {
            if let Some(root) = core::get_git_project_root_path(PathBuf::from(path)) {
                Ok(root.to_str().unwrap().to_owned())
            } else {
                Err(Error::external("Could not find git project root path."))
            }
        })
        .unwrap();
    ctx.globals().set("_get_git_project_root_path", f).unwrap();
}

pub fn _get_git_project_branches(ctx: &Context) {
    let f = ctx
        .create_function(|_, path: String| {
            if let Some(root) = core::get_git_project_branches(PathBuf::from(path)).ok() {
                Ok(root)
            } else {
                Err(Error::external("Could not find git branches."))
            }
        })
        .unwrap();
    ctx.globals().set("_get_git_project_branches", f).unwrap();
}