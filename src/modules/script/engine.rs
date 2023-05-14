use std::env;
use std::path::PathBuf;
use uuid::Uuid;

use rlua::Error as LuaError;
use rlua::FromLua;
use rlua::Table;
use rlua::{Context, Lua, Result, ToLua, UserData};
use tokio::runtime::Runtime;

use crate::modules::core;
use crate::modules::db::serializers::SerializableDateTime;
use crate::modules::db::DatabaseOperations;
use crate::modules::formats::text;
use crate::modules::formats::yaml::{get_yaml_value, update_yaml_value};
use crate::modules::log::ack;
use crate::modules::notes::markdown;
use crate::modules::notes::markdown::Page;
use crate::modules::projects::git::{GitOperations, SimpleRepo};
use crate::modules::projects::{agile, git};
use crate::modules::tasks::todoist;

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
    let todoist_sync = ctx
        .create_function(|_, db: String| {
            match env::var("TODOIST_TOKEN") {
                Ok(token) => {
                    todoist::init_db(&db);
                    Runtime::new()
                        .unwrap()
                        .block_on(todoist::sync(&token, &db))
                        .unwrap();
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
        .create_function(|ctx, (name, description, path): (String, String, String)| {
            agile::init_db(&path).ok().unwrap();
            let project = agile::Project {
                name,
                description,
                ..Default::default()
            };
            project.save(&path).ok().unwrap();
            let project_table = ctx.create_table().ok().unwrap();
            project_table
                .set("id", project.id.to_string())
                .ok()
                .unwrap();
            project_table.set("name", project.name).ok().unwrap();
            project_table
                .set("description", project.description)
                .ok()
                .unwrap();
            project_table
                .set("created_at", project.created_at.to_string())
                .ok()
                .unwrap();
            project_table
                .set("update_at", project.updated_at.to_string())
                .ok()
                .unwrap();

            Ok(project_table)
        })
        .unwrap();
    globals
        .set("agile_create_project", agile_create_project)
        .unwrap();
    let agile_create_sprint = ctx
        .create_function(
            |ctx, (project_id, name, start_date, path): (String, String, String, String)| {
                agile::init_db(&path).ok().unwrap();
                let sprint = agile::Sprint {
                    project_id: Uuid::parse_str(&project_id).unwrap(),
                    name,
                    start_date: SerializableDateTime::from_str(&start_date).unwrap(),
                    end_date: SerializableDateTime::from_str(&start_date)
                        .unwrap()
                        .add_weeks(3),
                    ..Default::default()
                };
                sprint.save(&path).ok().unwrap();
                let sprint_table = ctx.create_table().ok().unwrap();
                sprint_table.set("id", sprint.id.to_string()).ok().unwrap();
                sprint_table
                    .set("project_id", sprint.project_id.to_string())
                    .ok()
                    .unwrap();
                sprint_table.set("name", sprint.name).ok().unwrap();
                sprint_table
                    .set("start_date", sprint.start_date.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("end_date", sprint.end_date.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("created_at", sprint.created_at.to_string())
                    .ok()
                    .unwrap();
                sprint_table
                    .set("update_at", sprint.updated_at.to_string())
                    .ok()
                    .unwrap();
                Ok(sprint_table)
            },
        )
        .unwrap();
    globals
        .set("agile_create_sprint", agile_create_sprint)
        .unwrap();
}

/// Execute a script.
/// # Arguments
/// * `script` - The script to execute, as a `str`.
pub fn execute(script: &str) -> Result<()> {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        prepare_context(&lua_ctx);
        lua_ctx.load(&remove_comment_lines(script)).exec().unwrap();
        Ok(())
    })
}
