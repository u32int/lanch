use iced::widget::text;
use iced::Element;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use std::process::Command;

use super::*;

// I would use 'Application' but that is already taken by iced
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProgramSuggestion {
    name: String,
    exec: String,
    // TODO: icon
}

impl ProgramSuggestion {
    pub fn new(name: &str, exec: &str) -> Self {
        ProgramSuggestion {
            name: String::from(name),
            exec: String::from(exec),
        }
    }
}

impl Suggestion for ProgramSuggestion {
    fn view(&self) -> Element<SuggestionMessage> {
        text(format!("{}", self.name)).into()
    }

    fn execute(&self) {
        let mut exec = self.exec.split_whitespace();
        let mut cmd = Command::new(exec.nth(0).unwrap());
        // TODO: this causes some problems with certain args, as they are meant for shells. (ex. "%u")
        //exec.for_each(|arg| {
        //    cmd.arg(arg);
        //});
        cmd.spawn().expect("could not execute command.");
    }

    fn matches(&self, query: &String) -> bool {
        self.name.to_lowercase().contains(&query.to_lowercase())
    }
}

impl Display for ProgramSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
