use iced::widget::text;
use iced::Element;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use std::process::Command;

use super::*;

// I would use 'Application' but that is already taken by iced
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Program {
    name: String,
    exec: String,
    // TODO: icon
}

impl Program {
    pub fn new(name: &str, exec: &str) -> Self {
        Program {
            name: String::from(name),
            exec: String::from(exec),
        }
    }
}

impl Suggestion for Program {
    fn view(&self) -> Element<SuggestionMessage> {
        text(format!("Program: {}", self.name)).into()
    }

    fn execute(&self) {
        Command::new("sh")
            .arg("-c")
            .arg(&self.exec)
            .spawn()
            .expect("failed to start command.");
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
