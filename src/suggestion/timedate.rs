use chrono::Local;
use chrono_tz::Tz;

use iced::widget::text;

use std::cell::Cell;
use std::fmt::Display;

use super::*;

fn get_timezone(query: &String) -> Option<Tz> {
    for word in query.split(' ') {
        if word.is_empty() {
            continue;
        }

        // every (i think?) timezone name starts with a capital letter and this allows typing the
        // lowercase version and still getting the result
        let mut capitalized = String::from(word);
        // probably not a very optimal way to do this
        capitalized.replace_range(0..1, &capitalized.get(0..1).unwrap().to_uppercase());

        if let Ok(tz) = capitalized.parse::<Tz>() {
            return Some(tz);
        }

        // this shouldn't be _too_ expensive since .parse::<Tz>() uses a hashmap under the hood
        let prefixes = ["Africa", "Australia", "Asia", "Europe", "America", "Pacific"]; 
        for prefix in prefixes {
            if let Ok(tz) = format!("{prefix}/{capitalized}").parse::<Tz>() {
                return Some(tz);
            }
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

    fn matches(&self, query: &String) -> bool {
        if query.contains("time") {
            self.time_zone.set(get_timezone(query));
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

    fn matches(&self, query: &String) -> bool {
        if query.contains("date") {
            self.time_zone.set(get_timezone(query));
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
