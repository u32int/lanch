use std::cell::Cell;

use iced::{Element, Length, theme, Color};
use iced::widget::{container, text, row};
use super::{LanchMessage, ContainerBackgroundStyle, Lanch, SUGGESTIONS_PER_PAGE};

lazy_static::lazy_static! {
    static ref COLOR_INFO: iced::Color = Color::from_rgb8(51, 89, 218);
    static ref COLOR_WARN: iced::Color = Color::from_rgb8(201, 49, 22);
}

pub struct InfoBar {
    msg: Cell<Option<String>>,
    color: Option<Color>,
}

impl InfoBar {
    pub fn new() -> Self {
        Self { msg: Cell::new(None), color: None }
    }

    pub fn set_msg(&self, msg: Option<String>) {
        self.msg.replace(msg);
    }

    pub fn _set_color(&mut self, color: Color) {
        self.color.replace(color);
    }

    pub fn view(&self, upper: &Lanch) -> Element<LanchMessage> {
        // if there is a message available this clears it automatically so it only gets displayed
        // once
        if let Some(msg) = &self.msg.take() {
            let color = self.color.unwrap_or(*COLOR_WARN);

            container(text(msg))
                .width(Length::Fill)
                .style(theme::Container::Custom(Box::new(
                    ContainerBackgroundStyle::new(color),
                )))
        } else {
            container(row![text(format!(
                " Page: {} [{}-{}/{}]",
                upper.page,
                SUGGESTIONS_PER_PAGE * upper.page,
                SUGGESTIONS_PER_PAGE * upper.page + SUGGESTIONS_PER_PAGE.min(upper.suggestions.len()),
                upper.suggestions.len(),
            ))])
            .width(Length::Fill)
            .style(theme::Container::Custom(Box::new(
                ContainerBackgroundStyle::new(*COLOR_INFO),
            )))
        }.into()
    }
}
