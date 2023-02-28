use chrono::Local;

use iced::widget::text;

use std::fmt::Display;

use super::*;

#[derive(Debug)]
pub struct TimeSuggestion;

impl Suggestion for TimeSuggestion {
    fn view(&self) -> Element<SuggestionMessage> {
        let local = Local::now();

        text(local.format("%d of %b, %Y [UTC %z]")).into()
    }

    fn execute(&self) {} // TODO: copy to clipboard

    fn matches(&self, query: &String) -> bool {
        query.starts_with("time")
    }
}

impl Display for TimeSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time")
    }
}
