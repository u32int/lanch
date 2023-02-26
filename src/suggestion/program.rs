use iced::Element;
use iced::widget::text;
use std::fmt::Display;

use std::process::Command;

use super::*;

// I would use 'Application' but that is already taken by iced
pub struct Program {
    name: String,
    // TODO: icon
}

impl Program {
    pub fn with_name(name: &str) -> Self {
        Program {
            name: String::from(name),
        }
    }
}

impl Suggestion for Program {
    fn view(&self) -> Element<SuggestionMessage> {
        text(format!("Program: {}", self.name)).into()
    }

    fn execute(&self) {
        Command::new(&self.name).spawn().expect("failed to start command.");
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
