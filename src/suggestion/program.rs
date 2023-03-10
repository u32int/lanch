use iced::widget::{horizontal_space, image, row, svg, text};
use iced::{Element, Length};
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::process::Command;

use super::*;

// I would use 'Application' but that is already taken by iced
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProgramSuggestion {
    name: String,
    exec: String,
    icon: Option<PathBuf>,
}

impl ProgramSuggestion {
    pub fn new(name: &str, exec: &str, icon: Option<PathBuf>) -> Self {
        ProgramSuggestion {
            name: String::from(name),
            exec: String::from(exec),
            icon,
        }
    }
}

impl Suggestion for ProgramSuggestion {
    fn view(&self) -> Element<SuggestionMessage> {
        if let Some(path) = &self.icon {
            let img: Element<SuggestionMessage> = match path.extension() {
                Some(s) => match s.to_str() {
                    Some("png") | Some("jpg") => image(path).width(20).height(20).into(),
                    Some("svg") => svg::Svg::from_path(path).width(20).height(20).into(),
                    _ => return text(self.name.to_string()).into(),
                },
                _ => return text(self.name.to_string()).into(),
            };

            return row![
                img,
                horizontal_space(Length::Fixed(5f32)),
                text(self.name.to_string())
            ]
            .into();
        } else {
            return text(self.name.to_string()).into();
        }
    }

    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut exec = self.exec.split_whitespace();
        let mut cmd = Command::new(exec.next().unwrap());
        // TODO: this causes some problems with certain args, as they are meant for shells. (ex. "%u")
        //exec.for_each(|arg| {
        //    cmd.arg(arg);
        //});
        match cmd.spawn() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn matches(&self, query: &str) -> MatchLevel {
        if query == self.name {
            return MatchLevel::Exact;
        } else if self.name.to_lowercase().contains(&query.to_lowercase()) {
            return MatchLevel::Contained;
        }

        MatchLevel::NoMatch
    }
}

impl Display for ProgramSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program")
    }
}
