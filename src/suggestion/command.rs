use iced::widget::text;
use iced::Element;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use std::process::Command;

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandSuggestion {
    cmd: String,
}

impl CommandSuggestion {
    pub fn with_cmd(cmd: &str) -> Self {
        CommandSuggestion {
            cmd: String::from(cmd),
        }
    }
}

impl Suggestion for CommandSuggestion {
    fn view(&self) -> Element<LanchMessage> {
        text(format!("Command: \"{}\"", self.cmd)).into()
    }

    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>> {
        let mut exec = self.cmd.split_whitespace();
        let mut cmd = Command::new(exec.next().unwrap());
        exec.for_each(|arg| {
            cmd.arg(arg);
        });
        match cmd.spawn() {
            Ok(_) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn matches(&self, _query: &str) -> MatchLevel {
        MatchLevel::Exact
    }
}

impl Display for CommandSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command")
    }
}

pub struct CommandModule;

impl SuggestionModule for CommandModule {
    fn get_matches(&mut self, query: &str, v: &mut VecDeque<Rc<dyn Suggestion>>) {
        if query.starts_with('!') && query != "!" {
            v.push_front(Rc::new(CommandSuggestion::with_cmd(query.strip_prefix('!').unwrap())))
        }
    }
}
