use iced::widget::text;
use iced::Element;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use std::process::Command;

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutableSuggestion {
    name: String,
    exec: String,
}

impl ExecutableSuggestion {
    pub fn new(name: &str, exec: &str) -> Self {
        ExecutableSuggestion {
            name: String::from(name),
            exec: String::from(exec),
        }
    }
}

impl Suggestion for ExecutableSuggestion {
    fn view(&self) -> Element<SuggestionMessage> {
        text(self.name.to_string()).into()
    }

    fn execute(&self) {
        let mut exec = self.exec.split_whitespace();
        let mut cmd = Command::new(exec.next().unwrap());
        // TODO: this causes some problems with certain args, as they are meant for shells. (ex. "%u")
        //exec.for_each(|arg| {
        //    cmd.arg(arg);
        //});
        cmd.spawn().expect("could not execute command.");
    }

    fn matches(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase())
    }
}

impl Display for ExecutableSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
