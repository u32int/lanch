use iced::Element;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone)]
pub enum SuggestionMessage {}

/// Generic trait for things that are displayed in the suggestion list
pub trait Suggestion: Display + Debug {
    fn view(&self) -> Element<SuggestionMessage>;
    fn execute(&self);
    fn matches(&self, query: &String) -> bool;
}

pub mod program;
pub mod timedate;
pub mod help;
