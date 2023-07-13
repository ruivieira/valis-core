use std::env;
use std::path::PathBuf;

use rlua::{Context, Lua, Result, ToLua, UserData, Value};
use rlua::Error as LuaError;
use rlua::FromLua;
use rlua::Table;
use termion::color;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::modules::{core, db};
use crate::modules::db::DatabaseOperations;
use crate::modules::db::serializers::SerializableDateTime;
use crate::modules::formats::text;
use crate::modules::formats::yaml::{get_yaml_value, update_yaml_value};
use crate::modules::log::ack;
use crate::modules::notes::markdown;
use crate::modules::notes::markdown::Page;
use crate::modules::projects::{agile, git};
use crate::modules::projects::git::core::{GitOperations, SimpleRepo};
use crate::modules::tasks::todoist;

/// Remove she-bang comment lines from a script
fn remove_comment_lines(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim().starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n")
}

fn pretty_print_table(table: &Table, indent: usize) -> Result<()> {
    let pairs = table.clone().pairs::<Value, Value>();
    for pair in pairs {
        let (key, value) = pair?;
        print!("{}{}", " ".repeat(indent), color::Fg(color::Cyan)); // keys in cyan
        match key {
            Value::String(s) => print!("{}", s.to_str()?),
            Value::Integer(i) => print!("{}", i),
            Value::Number(n) => print!("{}", n),
            _ => print!("(non-string/number key)"),
        }
        print!(": {}", color::Fg(color::Reset));
        match value {
            Value::Table(t) => {
                println!();
                pretty_print_table(&t, indent + 2)?; // recursive call for nested tables
            }
            Value::String(s) => println!("{}{}", color::Fg(color::White), s.to_str()?), // strings in white
            Value::Integer(i) => println!("{}{}", color::Fg(color::Magenta), i), // integers in magenta
            Value::Number(n) => println!("{}{}", color::Fg(color::Magenta), n), // numbers in magenta
            _ => println!("(non-table/string/number value)"),
        }
        print!("{}", color::Fg(color::Reset));
    }
    Ok(())
}

/// Add built-in functions to the Lua `context`.
/// All functions are available in the global scope.
/// # Arguments
/// * `ctx` - The Lua context
pub fn prepare_context(ctx: &Context) {
    let globals = ctx.globals();
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
            return Ok(core::run_buffered(&command));
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
            return git::core::from_root(&path).ok_or_else(|| {
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
                    table
                        .set("link_type", wikilink.link_type as u8)
                        .ok()
                        .unwrap();
                    table
                })
                .collect::<Vec<Table>>();
            table.set("wikilinks", wikilinks).ok().unwrap();

            Ok(table)
        })
        .unwrap();
    globals.set("md_load", md_load).unwrap();
    let pprint = ctx
        .create_function(|_, table: Table| {
            pretty_print_table(&table, 2)
        })
        .unwrap();
    globals.set("pprint", pprint).unwrap();
    todoist::lua::todoist_sync(ctx);
    todoist::lua::todoist_add_task_to_sprint(ctx);
    agile::lua::agile_create_project(ctx);
    agile::lua::agile_create_sprint(ctx);
    agile::lua::agile_show_sprint(ctx);
    git::lua::_get_git_project_root_path(ctx);
    git::lua::_get_git_project_branches(ctx);
}

/// Execute a script.
/// # Arguments
/// * `script` - The script to execute, as a `str`.
pub fn execute(script: &str) -> Result<()> {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        prepare_context(&lua_ctx);
        let prelude = include_str!("prelude.lua");
        lua_ctx.load(prelude).exec().unwrap();
        lua_ctx.load(&remove_comment_lines(script)).exec().unwrap();
        Ok(())
    })
}
