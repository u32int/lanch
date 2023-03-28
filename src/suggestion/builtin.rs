use iced::widget::{column, text};
use std::fmt::Display;

use super::*;

#[derive(Debug)]
pub struct BuiltInSuggestion {
    name: String,
    execute_fn: fn() -> Result<Option<LanchMessage>, Box<dyn std::error::Error>>,
}

impl Suggestion for BuiltInSuggestion {
    fn view(&self) -> Element<LanchMessage> {
        column![text(&self.name),].into()
    }

    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>> {
        (self.execute_fn)()
    }

    fn matches(&self, query: &str) -> MatchLevel {
        if let Some(cmd) = query.strip_prefix('/') {
            if self.name == cmd {
                return MatchLevel::Exact;
            } else {
                return MatchLevel::Contained;
            }
        }

        MatchLevel::NoMatch
    }
}

impl Display for BuiltInSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "builtin cmd")
    }
}

pub struct BuiltInModule {
    cmds: Vec<Rc<BuiltInSuggestion>>,
}

impl BuiltInModule {
    pub fn new() -> Self {
        Self {
            cmds: vec![
                Rc::new(BuiltInSuggestion {
                    name: String::from("license"),
                    execute_fn: || Ok(Some(LanchMessage::SwitchLayout(crate::ui::Layout::License))),
                }),
                Rc::new(BuiltInSuggestion {
                    name: String::from("help"),
                    // execute_fn: || Ok(None),
                    execute_fn: || todo!(),
                }),
            ],
        }
    }
}

impl SuggestionModule for BuiltInModule {
    fn get_matches(&mut self, query: &str, v: &mut VecDeque<Rc<dyn Suggestion>>) {
        for cmd in &self.cmds {
            match cmd.matches(query) {
                MatchLevel::Exact => v.push_front(Rc::clone(cmd) as Rc<dyn Suggestion>),
                MatchLevel::Contained => v.push_back(Rc::clone(cmd) as Rc<dyn Suggestion>),
                MatchLevel::NoMatch => {}
            }
        }
    }
}
