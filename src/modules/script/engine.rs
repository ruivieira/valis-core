use rlua::{Context, Lua, Table};
use rlua::Error as LuaError;

use crate::modules::core;
use crate::modules::formats::text;
use crate::modules::formats::yaml::{get_yaml_value, update_yaml_value};
use crate::modules::log::ack;
use crate::modules::projects::git;
use crate::modules::projects::git::{GitOperations, SimpleRepo};

// pub trait FromLuaTable: Sized {
//     fn from_lua(lua_table: Table<'_>, ctx: Context<'_>) -> Result<Self, Err>;
// }

/// Remove she-bang comment lines from a script
fn remove_comment_lines(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim().starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Add built-in functions to the Lua `context`.
/// All functions are available in the global scope.
/// # Arguments
/// * `ctx` - The Lua context
pub fn prepare_context(ctx: &Context) {
    let globals = ctx.globals();
    globals.set("foo", 42).unwrap();
    let check_print = ctx
        .create_function(|_, message: String| {
            return Ok(format!("hello from rust! -> {}", &message));
        })
        .unwrap();
    globals.set("check_print", check_print).unwrap();
    let git_clone = ctx
        .create_function(|_, (url, destination): (String, String)| {
            let repo = SimpleRepo {
                url: url.to_owned(),
                branch: None,
                destination: destination.to_owned(),
            };
            ack(&format!(
                "cloning {} into {}",
                &url.to_owned(),
                &destination.to_owned()
            ));
            repo.clone();
            return Ok(());
        })
        .unwrap();
    globals.set("git_clone", git_clone).unwrap();
    let run = ctx
        .create_function(|_, command: String| {
            return Ok((core::run_buffered(&command)));
        })
        .unwrap();
    globals.set("run", run).unwrap();
    let get_yaml_value = ctx
        .create_function(|_, (file_path, yaml_key_path): (String, String)| {
            return Ok(get_yaml_value(&file_path, &yaml_key_path).ok().unwrap());
        })
        .unwrap();
    globals.set("yaml_get_value", get_yaml_value).unwrap();
    let update_yaml_value = ctx
        .create_function(
            |_, (file_path, yaml_key_path, new_value): (String, String, String)| {
                // TODO: Return the error properly
                let v = update_yaml_value(&file_path, &yaml_key_path, &new_value)
                    .ok()
                    .unwrap();
                return Ok(());
            },
        )
        .unwrap();
    globals.set("yaml_set_value", update_yaml_value).unwrap();
    let replace_matching_line = ctx
        .create_function(
            |_, (file_path, regex, new_value): (String, String, String)| {
                match text::replace_matching_line(&file_path, &regex, &new_value) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(LuaError::RuntimeError(
                        "Could not replace matching line".to_string(),
                    )),
                }
            },
        )
        .unwrap();
    globals.set("replace_line", replace_matching_line).unwrap();
    let set_dir = ctx
        .create_function(|_, dir: String| match core::set_dir(&dir) {
            Ok(()) => Ok(()),
            Err(e) => Err(LuaError::RuntimeError(
                "Could not set current directory".to_string(),
            )),
        })
        .unwrap();
    globals.set("set_dir", set_dir).unwrap();
    let get_dir = ctx
        .create_function(|_, ()| match core::get_dir() {
            Ok(dir) => Ok(dir),
            Err(e) => Err(LuaError::RuntimeError(
                "Could not get current directory".to_string(),
            )),
        })
        .unwrap();
    globals.set("get_dir", get_dir).unwrap();
    let from_home = ctx
        .create_function(|_, path: String| {
            return core::from_home(&path)
                .ok_or_else(|| LuaError::RuntimeError("Could not get path from home".to_string()));
        })
        .unwrap();
    globals.set("from_home", from_home).unwrap();
    let git_from_root = ctx
        .create_function(|_, path: String| {
            return git::from_root(&path).ok_or_else(|| {
                LuaError::RuntimeError("Could not get path from git root".to_string())
            });
        })
        .unwrap();
    globals.set("git_from_root", git_from_root).unwrap();
}

/// Execute a script.
/// # Arguments
/// * `script` - The script to execute, as a `str`.
pub fn execute(script: &str) -> Result<(), rlua::Error> {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        prepare_context(&lua_ctx);
        lua_ctx.load(&remove_comment_lines(script)).exec().unwrap();
        Ok(())
    })
}
