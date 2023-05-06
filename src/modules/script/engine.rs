use rlua::{Context, Lua};

use crate::modules::core;
use crate::modules::log::ack;
use crate::modules::projects::git::{GitOperations, SimpleRepo};
use crate::modules::software;

fn remove_comment_lines(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim().starts_with("#"))
        .collect::<Vec<&str>>()
        .join("\n")
}

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
}

pub fn execute(script: &str) -> Result<(), rlua::Error> {
    let lua = Lua::new();

    lua.context(|lua_ctx| {
        prepare_context(&lua_ctx);
        lua_ctx.load(&remove_comment_lines(script)).exec().unwrap();
        Ok(())
    })
}