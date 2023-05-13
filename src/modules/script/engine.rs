use std::{env, fs, io};
use std::path::PathBuf;

use rlua::{Context, Lua, Table, ToLua, ToLuaMulti};
use rlua::Error as LuaError;
use tokio::runtime::Runtime;

use crate::modules::core;
use crate::modules::db::DatabaseOperations;
use crate::modules::formats::text;
use crate::modules::formats::yaml::{get_yaml_value, update_yaml_value};
use crate::modules::log::ack;
use crate::modules::notes::markdown;
use crate::modules::notes::markdown::Page;
use crate::modules::projects::{agile, git};
use crate::modules::projects::agile::Project;
use crate::modules::projects::git::{GitOperations, SimpleRepo};
use crate::modules::tasks::todoist;

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
    let md_load = ctx
        .create_function(|ctx, path: String| {
            let mut pb = PathBuf::new();
            pb.push(path);
            let page: Page = markdown::PageLoader::from_path(&pb);
            let table = ctx.create_table()?;
            table.set("title", page.title.to_string())?;
            table.set("path", page.path.to_str().unwrap_or(""))?;
            table.set("contents", page.contents.to_string())?;
            let wikilinks = page
                .wikilinks
                .into_iter()
                .map(|wikilink| {
                    let table = ctx.create_table().ok().unwrap();
                    table.set("name", wikilink.name).ok().unwrap();
                    table.set("link", wikilink.link).ok().unwrap();
                    table.set("anchor", wikilink.anchor).ok().unwrap();
                    table.set("link_type", wikilink.link_type as u8).ok().unwrap();
                    table
                }).collect::<Vec<Table>>();
            table.set("wikilinks", wikilinks).ok().unwrap();

            Ok(table)
        }).unwrap();
    globals.set("md_load", md_load).unwrap();
    let todoist_sync = ctx
        .create_function(|_, db: String| {
            match env::var("TODOIST_TOKEN") {
                Ok(token) => {
                    todoist::init_db(&db);
                    Runtime::new().unwrap().block_on(todoist::sync(&token, &db)).unwrap();
                }
                Err(e) => {
                    println!("Failed to read TODOIST_TOKEN: {}", e);
                }
            }
            Ok(())
        })
        .unwrap();
    globals.set("todoist_sync", todoist_sync).unwrap();
    let agile_create_project = ctx
        .create_function(|_, (name, description, path): (String, String, String)| {
            agile::init_db(&path);
            let project = agile::Project {
                name: name,
                description: description,
                ..Default::default()
            };
            project.save(&path);
            Ok(())
        })
        .unwrap();
    globals.set("agile_create_project", agile_create_project).unwrap();
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
