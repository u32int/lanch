use chrono::Local;
use chrono_tz::Tz;

use iced::widget::text;

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
    fn view(&self) -> Element<SuggestionMessage> {
        let now = Local::now();

        if let Some(tz) = self.time_zone.get() {
            let tz_now = now.with_timezone(&tz);
            text(format!("{}: {}", tz.name(), tz_now.format("%H:%M:%S")))
        } else {
            text(now.format("Local time: %H:%M:%S"))
        }
        .into()
    }

    fn execute(&self) {} // TODO: copy to clipboard

    fn matches(&self, query: &str) -> bool {
        if query.contains("time") {
            self.time_zone.set(get_timezone(query, "time"));
            return true;
        }
        false
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
    fn view(&self) -> Element<SuggestionMessage> {
        let now = Local::now();

        if let Some(tz) = self.time_zone.get() {
            let tz_now = now.with_timezone(&tz);
            text(format!(
                "Date [{}]: {}",
                tz.name(),
                tz_now.format("%d of %B %Y")
            ))
        } else {
            text(now.format("Date: %d of %B %Y"))
        }
        .into()
    }

    fn execute(&self) {} // TODO: copy to clipboard

    fn matches(&self, query: &str) -> bool {
        if query.contains("date") {
            self.time_zone.set(get_timezone(query, "date"));
            return true;
        }
        false
    }
}

impl Display for DateSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Date")
    }
}
