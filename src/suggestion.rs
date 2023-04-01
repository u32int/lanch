use iced::Element;
use std::rc::Rc;
use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use crate::ui::LanchMessage;

pub enum MatchLevel {
    Exact,
    Contained,
    NoMatch
}

/// Generic trait for things that are displayed in the suggestion list
pub trait Suggestion: Display + Debug {
    // display the element as a collection of iced widgets
    fn view(&self) -> Element<LanchMessage>;

    // triggered when the user presses enter on the selected item
    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>>;

    // condition checked to decide whether or not to display the suggestion based on the query
    fn matches(&self, query: &str) -> MatchLevel;
}

/// Suggestion modules add matching modules to the suggestion list based on the passed query
pub trait SuggestionModule {
    fn get_matches(&mut self, query: &str, v: &mut VecDeque<Rc<dyn Suggestion>>);
}

pub mod executable;
pub mod timedate;
pub mod command;
pub mod builtin;
