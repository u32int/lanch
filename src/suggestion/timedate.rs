use chrono::Local;
use chrono_tz::Tz;

use iced::widget::{column, horizontal_space, row, text, vertical_space};
use iced::Length;

use std::cell::Cell;
use std::fmt::Display;

use super::*;

fn get_timezone(query: &str, keyword: &str) -> Option<Tz> {
    let mut tz_string = query.to_string();

    let keyword_idx = query.find(keyword).unwrap();
    tz_string.replace_range(keyword_idx..keyword_idx + keyword.len(), "");
    let tz_string = tz_string.trim();

    if tz_string.is_empty() {
        return None;
    }

    let tz_string: String = tz_string
        .split(' ')
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(i, word)| {
            format!(
                "{}{}{}",
                if i != 0 { "_" } else { "" },
                word.get(0..1).unwrap().to_uppercase(),
                word.get(1..).unwrap()
            )
        })
        .collect();

    if let Ok(tz) = tz_string.parse::<Tz>() {
        return Some(tz);
    }

    // this shouldn't be _too_ expensive since .parse::<Tz>() uses a hashmap under the hood
    let prefixes = [
        "Africa",
        "Australia",
        "Asia",
        "Europe",
        "America",
        "Pacific",
    ];
    for prefix in prefixes {
        if let Ok(tz) = format!("{prefix}/{tz_string}").parse::<Tz>() {
            return Some(tz);
        }
    }

    None
}

/// Display the time
#[derive(Debug, Default)]
pub struct TimeSuggestion {
    time_zone: Cell<Option<Tz>>,
}

impl Suggestion for TimeSuggestion {
    fn view(&self) -> Element<LanchMessage> {
        let now = Local::now();

        let txt = if let Some(tz) = self.time_zone.get() {
            let tz_now = now.with_timezone(&tz);
            text(format!("{}: {}", tz.name(), tz_now.format("%H:%M:%S")))
        } else {
            text(now.format("Local time: %H:%M:%S"))
        };

        column![
            vertical_space(Length::Fixed(10f32)),
            row![
                horizontal_space(Length::Fixed(8f32)),
                txt,
                horizontal_space(Length::Fixed(8f32)),
            ],
            vertical_space(Length::Fixed(10f32)),
        ]
        .into()
    }

    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>> {
        Ok(None)
    } // TODO: copy to clipboard

    fn matches(&self, query: &str) -> MatchLevel {
        if query.contains("time") {
            let tz = get_timezone(query, "time");
            self.time_zone.set(tz);

            return if tz.is_some() || query == "time" {
                MatchLevel::Exact
            } else {
                MatchLevel::Contained
            };
        }

        MatchLevel::NoMatch
    }
}

impl Display for TimeSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time")
    }
}

/// Display the Date
#[derive(Debug, Default)]
pub struct DateSuggestion {
    time_zone: Cell<Option<Tz>>,
}

impl Suggestion for DateSuggestion {
    fn view(&self) -> Element<LanchMessage> {
        let now = Local::now();

        let txt = if let Some(tz) = self.time_zone.get() {
            let tz_now = now.with_timezone(&tz);
            text(format!(
                "Date [{}]: {}",
                tz.name(),
                tz_now.format("%d of %B %Y")
            ))
        } else {
            text(now.format("Date: %d of %B %Y"))
        };

        column![
            vertical_space(Length::Fixed(10f32)),
            row![
                horizontal_space(Length::Fixed(8f32)),
                txt,
                horizontal_space(Length::Fixed(8f32)),
            ],
            vertical_space(Length::Fixed(10f32)),
        ]
        .into()
    }

    fn execute(&self) -> Result<Option<LanchMessage>, Box<dyn std::error::Error>> {
        Ok(None)
    } // TODO: copy to clipboard

    fn matches(&self, query: &str) -> MatchLevel {
        if query.contains("date") {
            let tz = get_timezone(query, "date");
            self.time_zone.set(tz);

            return if tz.is_some() || query == "date" {
                MatchLevel::Exact
            } else {
                MatchLevel::Contained
            };
        }

        MatchLevel::NoMatch
    }
}

impl Display for DateSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Date")
    }
}

pub struct TimeDateModule {
    time: Rc<TimeSuggestion>,
    date: Rc<DateSuggestion>,
}

impl TimeDateModule {
    pub fn new() -> Self {
        Self {
            time: Rc::new(TimeSuggestion::default()),
            date: Rc::new(DateSuggestion::default()),
        }
    }
}

impl SuggestionModule for TimeDateModule {
    fn get_matches(&mut self, query: &str, v: &mut VecDeque<Rc<dyn Suggestion>>) {
        // Only exact matches for now, could be improved by trying to guess the locations etc.
        match self.time.matches(query) {
            MatchLevel::Exact => v.push_front(Rc::clone(&self.time) as Rc<dyn Suggestion>),
            MatchLevel::Contained => v.push_back(Rc::clone(&self.time) as Rc<dyn Suggestion>),
            MatchLevel::NoMatch => {}
        }

        match self.date.matches(query) {
            MatchLevel::Exact => v.push_front(Rc::clone(&self.date) as Rc<dyn Suggestion>),
            MatchLevel::Contained => v.push_back(Rc::clone(&self.date) as Rc<dyn Suggestion>),
            MatchLevel::NoMatch => {}
        }
    }
}
