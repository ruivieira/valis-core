use std::borrow::Cow::{self, Borrowed, Owned};

use rlua::Lua;
use rustyline::{CompletionType, Config, EditMode, Editor, Helper};
use rustyline::completion::{Candidate, Completer};
use rustyline::Context;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use crate::modules::script::engine::prepare_context;

struct CustomCompleter {
    commands: Vec<&'static str>,
}

impl Completer for CustomCompleter {
    type Candidate = SimpleCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<SimpleCandidate>), rustyline::error::ReadlineError> {
        let mut candidates = Vec::new();
        for command in &self.commands {
            if command.starts_with(line) {
                candidates.push(SimpleCandidate {
                    display: command.to_string(),
                    replacement: command[pos..].to_string(),
                });
            }
        }

        Ok((0, candidates))
    }
}

#[derive(Debug, Clone)]
struct SimpleCandidate {
    display: String,
    replacement: String,
}

impl Candidate for SimpleCandidate {
    fn display(&self) -> &str {
        &self.display
    }

    fn replacement(&self) -> &str {
        &self.replacement
    }
}

struct CustomHelper {
    completer: CustomCompleter,
}

impl Helper for CustomHelper {}

impl Completer for CustomHelper {
    type Candidate = SimpleCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<SimpleCandidate>), rustyline::error::ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for CustomHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CustomHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Borrowed(hint)
    }
}

impl Validator for CustomHelper {}

pub fn repl() {
    let config = Config::builder()
        .edit_mode(EditMode::Emacs)
        .completion_type(CompletionType::List)
        .build();

    let commands = vec!["git_clone", "run"];
    let completer = CustomCompleter { commands };

    let helper = CustomHelper { completer };
    let mut editor = Editor::with_config(config);
    editor.set_helper(Some(helper));

    let lua = Lua::new();
    lua.context(|lua_ctx| {
        prepare_context(&lua_ctx);
    });

    loop {
        let readline = editor.readline(">> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str());
                if line.trim() == "quit" {
                    break;
                }
                // Execute the Lua code
                lua.context(|lua_ctx| {
                    match lua_ctx.load(line.as_str()).exec() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                });
            }
            Err(_) => break,
        }
    }
}
