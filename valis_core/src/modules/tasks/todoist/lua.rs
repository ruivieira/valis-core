use std::env;

use rlua::{Context, Error};
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::modules::db;
use crate::modules::tasks::todoist;
use crate::modules::tasks::todoist::core::add_task_to_sprint;

pub fn todoist_sync(ctx: &Context) {
    let f = ctx
        .create_function(|_, db: String| {
            match env::var("TODOIST_TOKEN") {
                Ok(token) => {
                    db::init_db(&db);
                    Runtime::new()
                        .unwrap()
                        .block_on(todoist::core::sync(&token, &db))
                        .unwrap();
                }
                Err(e) => {
                    println!("Failed to read TODOIST_TOKEN: {}", e);
                }
            }
            Ok(())
        })
        .unwrap();
    ctx.globals().set("todoist_sync", f).unwrap();
}

pub fn todoist_add_task_to_sprint(ctx: &Context) {
    let f = ctx
        .create_function(|_, (sprint_id, task_id, db): (String, String, String)| {
            match Uuid::parse_str(&sprint_id) {
                Ok(sprint_uuid) => {
                    println!("Sprint id: {}", sprint_id);
                    match add_task_to_sprint(&db, &sprint_uuid, task_id) {
                        Ok(()) => Ok(()),
                        Err(e) => Err(Error::RuntimeError(
                            "Could not save task to Sprint".to_string(),
                        )),
                    }
                }
                Err(e) => Err(Error::RuntimeError(
                    "Error parsiong Sprint UUID".to_string(),
                )),
            }
        })
        .unwrap();
    ctx.globals().set("todoist_add_task_to_sprint", f).unwrap();
}
