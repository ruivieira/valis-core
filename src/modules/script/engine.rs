use rlua::Lua;

use crate::modules::log::ack;
use crate::modules::projects::git::{GitOperations, SimpleRepo};

fn remove_comment_lines(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim().starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n")
}

pub fn execute(script: &str) -> Result<(), rlua::Error> {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals.set("foo", 42).unwrap();
        let check_print =
            lua_ctx.create_function(|_, (message): (String)| {
                return Ok(format!("hello from rust! -> {}", &message));
            })?;
        globals.set("check_print", check_print)?;
        let git_clone =
            lua_ctx.create_function(|_, (url, destination): (String, String)| {
                let repo = SimpleRepo {
                    url: url.to_owned(),
                    branch: None,
                    destination: destination.to_owned(),
                };
                ack(&format!("cloning {} into {}", &url.to_owned(), &destination.to_owned()));
                repo.clone();
                return Ok(());
            })?;
        globals.set("git_clone", git_clone)?;
        lua_ctx.load(&remove_comment_lines(script)).exec().unwrap();
        Ok(())
    })
}