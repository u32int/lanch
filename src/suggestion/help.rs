use iced::widget::{text, column};
use std::fmt::Display;

use super::*;

#[derive(Debug)]
pub struct HelpSuggestion;

impl Suggestion for HelpSuggestion {
    fn view(&self) -> Element<SuggestionMessage> {
        // TODO: help styling (bold, different fonts etc)
        // (maybe make this element unselectable somehow?)
        //
        // TODO: help should display if modules are loaded.
        // probably best to make it a field on this struct and pass a list of them on init
        column![
            text("?      this menu"),
            text("time      display local time"),
        ].into()
    }

    fn execute(&self)-> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    } 

    fn matches(&self, query: &str) -> MatchLevel {
        if query == "?" || query == "help" {
            return MatchLevel::Exact
        }

        MatchLevel::NoMatch
    }
}

impl Display for HelpSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Help")
    }
}
