use rlua::{Context, Lua};

use crate::modules::core;
use crate::modules::formats::yaml::{get_yaml_value, update_yaml_value};
use crate::modules::log::ack;
use crate::modules::projects::git::{GitOperations, SimpleRepo};
use crate::modules::software;

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
    let check_print =
        ctx.create_function(|_, (message): (String)| {
            return Ok(format!("hello from rust! -> {}", &message));
        }).unwrap();
    globals.set("check_print", check_print).unwrap();
    let git_clone =
        ctx.create_function(|_, (url, destination): (String, String)| {
            let repo = SimpleRepo {
                url: url.to_owned(),
                branch: None,
                destination: destination.to_owned(),
            };
            ack(&format!("cloning {} into {}", &url.to_owned(), &destination.to_owned()));
            repo.clone();
            return Ok(());
        }).unwrap();
    globals.set("git_clone", git_clone).unwrap();
    let run =
        ctx.create_function(|_, (command): (String)| {
            return Ok((core::run_buffered(&command)));
        }).unwrap();
    globals.set("run", run).unwrap();
    let get_yaml_value =
        ctx.create_function(|_, (file_path, yaml_key_path): (String, String)| {
            return Ok(get_yaml_value(&file_path, &yaml_key_path).ok().unwrap());
        }).unwrap();
    globals.set("yaml_get_value", get_yaml_value).unwrap();
    let update_yaml_value =
        ctx.create_function(|_, (file_path, yaml_key_path, new_value): (String, String, String)| {
            // TODO: Return the error properly
            let v = update_yaml_value(&file_path, &yaml_key_path, &new_value).ok().unwrap();
            return Ok(());
        }).unwrap();
    globals.set("yaml_set_value", update_yaml_value).unwrap();
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