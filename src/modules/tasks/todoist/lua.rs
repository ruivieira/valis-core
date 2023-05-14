use std::env;

use rlua::Context;
use tokio::runtime::Runtime;

use crate::modules::db;
use crate::modules::tasks::todoist;

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
