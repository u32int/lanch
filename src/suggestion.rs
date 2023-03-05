use iced::Element;
use std::fmt::{Debug, Display};

pub enum MatchLevel {
    Exact,
    Contained,
    NoMatch
}

#[derive(Debug, Clone)]
pub enum SuggestionMessage {}

/// Generic trait for things that are displayed in the suggestion list
pub trait Suggestion: Display + Debug {
    // display the element as a collection of iced widgets
    fn view(&self) -> Element<SuggestionMessage>;

    // triggered when the user presses enter on the selected item
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>>;

    // condition checked to decide whether or not to display the suggestion based on the query
    fn matches(&self, query: &str) -> MatchLevel;
}

pub mod executable;
pub mod help;
pub mod program;
pub mod timedate;
pub mod command;
