// TODO: consider merging programs and executables into one thing now that we have modules?
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
    fn view(&self) -> Element<LanchMessage> {
        text(format!("{} [{}]", self.name, self.exec)).into()
    }

    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>> {
        let mut exec = self.exec.split_whitespace();
        let mut cmd = Command::new(exec.next().unwrap());

        match cmd.spawn() {
            Ok(_) => Ok(None),
            Err(e) => Err(Box::new(e))
        }
    }

    fn matches(&self, query: &str) -> MatchLevel {
        if query == self.name {
            return MatchLevel::Exact
        } else if self.name.to_lowercase().contains(&query.to_lowercase()) {
            return MatchLevel::Contained
        }

        MatchLevel::NoMatch
    }
}

impl Display for ExecutableSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Executable")
    }
}

pub struct ExecutableModule {
    cache: Vec<Rc<ExecutableSuggestion>>,
}

impl ExecutableModule {
    pub fn new(cache: Vec<Rc<ExecutableSuggestion>>) -> Self {
        Self { cache }
    }
}

impl SuggestionModule for ExecutableModule {
    fn get_matches(&mut self, query: &str, v: &mut VecDeque<Rc<dyn Suggestion>>) {
        // TODO: we can do better than this efficiency wise
        self.cache.iter().for_each(|p| match p.matches(query) {
            MatchLevel::Exact => v.push_front(Rc::clone(&p) as Rc<dyn Suggestion>),
            MatchLevel::Contained => v.push_back(Rc::clone(&p) as Rc<dyn Suggestion>),
            MatchLevel::NoMatch => {}
        })
    }
}
