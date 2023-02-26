use std::fmt::Display;
use iced::Element;

#[derive(Debug, Clone)]
pub enum SuggestionMessage {}

pub trait Suggestion: Display {
    fn view(&self) -> Element<SuggestionMessage>;

    fn execute(&self);
}

pub mod program;
