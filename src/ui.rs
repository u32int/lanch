use iced::widget::{
    column, container, horizontal_rule, horizontal_space, row, text, text_input, vertical_space,
};
use iced::{
    alignment, executor, keyboard, subscription, theme, window, Application, Background, Color,
    Command, Element, Event, Length, Theme,
};

use std::collections::VecDeque;
use std::rc::Rc;

mod settings;
mod infobar;

use super::cache;
use crate::suggestion::*;
use settings::*;

pub fn init(cache: cache::LanchCache) -> iced::Result {
    Lanch::run(settings::settings(cache))
}

lazy_static::lazy_static! {
    static ref QUERY_INPUT_ID: text_input::Id = text_input::Id::unique();

    static ref COLOR_INFO: iced::Color = Color::from_rgb8(51, 89, 218);
    static ref COLOR_WARN: iced::Color = Color::from_rgb8(201, 49, 22);
}

const SUGGESTIONS_PER_PAGE: usize = 10;

pub struct Lanch {
    // initialized from flags, contain the default values for window settings etc.
    options: LanchOptions,

    // cached programs to limit potentially hundreds of fs calls to 1 cache.
    // TODO: refresh periodically
    program_cache: Vec<Rc<dyn Suggestion>>,
    executable_cache: Vec<Rc<dyn Suggestion>>,

    // loaded modules providing extra suggestion functionality
    modules: Vec<Rc<dyn Suggestion>>,

    // the current query in the text box
    query: String,

    // suggestions displayed to the user
    suggestions: VecDeque<Rc<dyn Suggestion>>,

    // the currently selected suggestion
    selected: usize,

    // the current page of suggestions
    page: usize,

    // application theme
    theme: Theme,

    // the bottom info bar 
    info_bar: infobar::InfoBar,
}

// Could possibly be extended for grid layouts
#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum LanchMessage {
    QueryChanged(String),
    NavigateList(Direction),
    ExecuteSelected,
    Escape,
    SuggestionMessage(SuggestionMessage),
}

impl Application for Lanch {
    type Message = LanchMessage;
    type Executor = executor::Default;
    type Flags = LanchFlags;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Lanch, Command<Self::Message>) {
        (
            Lanch {
                options: flags.options,
                program_cache: flags
                    .cache
                    .programs
                    .into_iter()
                    .map(|elem| Rc::new(elem) as Rc<dyn Suggestion>)
                    .collect(),
                executable_cache: flags
                    .cache
                    .executables
                    .into_iter()
                    .map(|elem| Rc::new(elem) as Rc<dyn Suggestion>)
                    .collect(),
                modules: vec![
                    Rc::new(timedate::TimeSuggestion::default()),
                    Rc::new(timedate::DateSuggestion::default()),
                    Rc::new(help::HelpSuggestion),
                ], // temporary, will load from config eventually
                query: String::new(),
                suggestions: VecDeque::new(),
                selected: 0,
                page: 0,
                theme: Theme::Dark,
                info_bar: infobar::InfoBar::new(),
            },
            Command::batch(vec![
                window::gain_focus(),
                text_input::focus(QUERY_INPUT_ID.clone()),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("lanch")
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            LanchMessage::QueryChanged(q) => {
                self.query = q.trim_start().to_string();
                self.selected = 0;
                self.page = 0;
                self.generate_suggestions();

                if self.suggestions.is_empty() {
                    return window::resize(
                        self.options.window_size.0,
                        self.options.font_size as u32 * 5,
                    );
                } else {
                    // TODO: as of right now I haven't figured out a way to get the actual height
                    // of what is rendered so we kinda "guess" with font size and the number of
                    // elements. Works ok-ish for now, though problems start when the suggestions
                    // are not the same height (like date/time).
                    return window::resize(
                        self.options.window_size.0,
                        self.options.window_size.1.min(
                            (self.suggestions.len().min(SUGGESTIONS_PER_PAGE) + 5)
                                .saturating_mul(self.options.font_size as usize + 2)
                                as u32,
                        ),
                    );
                }
            }
            LanchMessage::NavigateList(d) => match d {
                Direction::Up => {
                    if self.selected == 0 && self.page != 0 {
                        self.page -= 1;
                        self.selected = SUGGESTIONS_PER_PAGE - 1;
                    } else {
                        self.selected = (self.selected.saturating_sub(1))
                            .clamp(0, self.suggestions.len() - self.page * SUGGESTIONS_PER_PAGE);
                    }
                }
                Direction::Down => {
                    if self.selected == SUGGESTIONS_PER_PAGE - 1
                        && self.page < self.suggestions.len() / SUGGESTIONS_PER_PAGE
                    {
                        self.selected = 0;
                        self.page += 1;
                    } else {
                        self.selected = (self.selected + 1).clamp(
                            0,
                            SUGGESTIONS_PER_PAGE
                                .min(self.suggestions.len() - self.page * SUGGESTIONS_PER_PAGE)
                                - 1,
                        );
                    }
                }
            },
            LanchMessage::ExecuteSelected => {
                match self.suggestions.get(self.selected).unwrap().execute() {
                    Ok(()) => return window::close(),
                    Err(e) => {
                        self.info_bar.set_msg(Some(format!(" Error: {}", e)));
                    }
                }
            }
            LanchMessage::Escape => {
                return window::close();
            }
            _ => todo!(),
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let input = text_input("Search...", &self.query, LanchMessage::QueryChanged)
            .id(QUERY_INPUT_ID.clone())
            .width(Length::Fill);

        let suggestions = container(self.view_suggestions());

        container(column![
            // search box
            input,
            horizontal_rule(3),
            vertical_space(Length::Fixed(5f32)),
            // suggestions
            suggestions.width(Length::Fill),
            vertical_space(Length::Fill),
            self.info_bar.view(self),
        ])
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        subscription::events_with(|event, _status| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) => Self::handle_key(key_code, modifiers),
            _ => None,
        })
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}

impl Lanch {
    // Updates the suggestions field based on the query
    fn generate_suggestions(&mut self) {
        self.suggestions.clear();

        if self.query.is_empty() {
            return;
        }

        let trimmed_query = self.query.trim();
        let mut match_counter: usize = 0;

        let mut add_suggestions = |vec: &Vec<Rc<dyn Suggestion>>| {
            for elem in vec {
                match elem.matches(trimmed_query) {
                    MatchLevel::Contained => self.suggestions.push_back(Rc::clone(elem)),
                    MatchLevel::Exact => self.suggestions.push_front(Rc::clone(elem)),
                    MatchLevel::NoMatch => continue,
                }

                match_counter += 1;
            }
        };

        add_suggestions(&self.modules);
        add_suggestions(&self.program_cache);
        add_suggestions(&self.executable_cache);

        // CommandSuggestion - just run the provided query as a command
        if let Some(cmd) = trimmed_query.strip_prefix('!') {
            self.suggestions
                .push_front(Rc::new(command::CommandSuggestion::with_cmd(
                    cmd,
                )))
        } else {
            self.suggestions
                .push_back(Rc::new(command::CommandSuggestion::with_cmd(
                    trimmed_query,
                )))
        }
    }

    // Turns the suggestion field into widgets
    fn view_suggestions(&self) -> Element<LanchMessage> {
        if self.suggestions.is_empty() {
            let info = if self.query.is_empty() {
                "Enter a query to get suggestions."
            } else {
                "No matches."
            };

            return text(info)
                .style(Color::from([0.7, 0.7, 0.7]))
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .into();
        }

        //println!("RENDERING STRUCTURE\n{:#?}", self.suggestions);

        let list = column(
            // Display all the suggestions
            self.suggestions
                .iter()
                .skip(SUGGESTIONS_PER_PAGE * self.page)
                .take(SUGGESTIONS_PER_PAGE)
                .enumerate()
                .map(|(i, sg)| {
                    let elem = if i == self.selected {
                        container(sg.view().map(LanchMessage::SuggestionMessage))
                            .style(theme::Container::Custom(Box::new(
                                ContainerBackgroundStyle::new(Color::from([0.3, 0.3, 0.3])),
                            )))
                            .into()
                    } else {
                        sg.view().map(LanchMessage::SuggestionMessage)
                    };

                    row![
                        horizontal_space(Length::Fixed(10f32)),
                        elem,
                        horizontal_space(Length::Fill),
                        text(sg.to_string())
                            .style(theme::Text::Color(Color::from([0.5, 0.5, 0.5]))),
                        horizontal_space(Length::Fixed(20f32)),
                    ]
                    .into()
                })
                .collect(),
        )
        .spacing(5)
        .width(Length::Fill);

        list.into()
    }

    fn handle_key(
        key_code: keyboard::KeyCode,
        modifiers: keyboard::Modifiers,
    ) -> Option<LanchMessage> {
        use keyboard::{KeyCode, Modifiers};

        match key_code {
            KeyCode::Down => Some(LanchMessage::NavigateList(Direction::Down)),
            KeyCode::J if modifiers == Modifiers::CTRL => {
                Some(LanchMessage::NavigateList(Direction::Down))
            }
            KeyCode::Up => Some(LanchMessage::NavigateList(Direction::Up)),
            KeyCode::K if modifiers == Modifiers::CTRL => {
                Some(LanchMessage::NavigateList(Direction::Up))
            }

            KeyCode::Enter => Some(LanchMessage::ExecuteSelected),
            KeyCode::Escape => Some(LanchMessage::Escape),
            _ => None,
        }
    }
}

// TODO: move
struct ContainerBackgroundStyle {
    color: iced::Color,
}

impl ContainerBackgroundStyle {
    fn new(color: iced::Color) -> Self {
        Self { color }
    }
}

impl container::StyleSheet for ContainerBackgroundStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.color)),
            text_color: Some(style.palette().text),
            ..Default::default()
        }
    }
}
