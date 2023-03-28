use iced::widget::{
    column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input,
    vertical_space,
};
use iced::{
    alignment, executor, keyboard, subscription, theme, window, Application, Background, Color,
    Command, Element, Event, Length, Theme,
};

use std::collections::VecDeque;
use std::rc::Rc;

mod infobar;
mod settings;

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

    // The current display layout of the application
    layout: Layout,

    // loaded modules providing extra suggestion functionality
    modules: Vec<Box<dyn SuggestionModule>>,

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    Default,
    License,
    Help,
}

#[derive(Debug, Clone)]
pub enum LanchMessage {
    QueryChanged(String),
    NavigateList(Direction),
    ExecuteSelected,
    Escape,
    SwitchLayout(Layout),
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
                modules: vec![
                    Box::new(builtin::BuiltInModule::new()),
                    Box::new(command::CommandModule),
                    Box::new(timedate::TimeDateModule::new()),
                    Box::new(program::ProgramModule::new(
                        flags
                            .cache
                            .programs
                            .into_iter()
                            .map(|x| Rc::new(x))
                            .collect(),
                    )),
                    Box::new(executable::ExecutableModule::new(
                        flags
                            .cache
                            .executables
                            .into_iter()
                            .map(|x| Rc::new(x))
                            .collect(),
                    )),
                ], // temporary, will load from config eventually
                layout: Layout::Default,
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
                if let Some(sel) = self
                    .suggestions
                    .get(self.selected + self.page * SUGGESTIONS_PER_PAGE)
                {
                    match sel.execute() {
                        Ok(Some(msg)) => return self.update(msg),
                        Ok(None) => return window::close(),
                        Err(e) => {
                            self.info_bar.set_msg(Some(format!(" Error: {}", e)));
                        }
                    }
                }
            }
            LanchMessage::Escape => match self.layout {
                Layout::Help | Layout::License => {
                    return Command::batch(vec![
                        self.update(LanchMessage::SwitchLayout(Layout::Default)),
                        text_input::focus(QUERY_INPUT_ID.clone()),
                    ])
                }
                Layout::Default => return window::close(),
            },
            LanchMessage::SwitchLayout(layout) => {
                self.layout = layout;

                match layout {
                    Layout::Default => {
                        return window::resize(
                            self.options.window_size.0,
                            self.options.window_size.1,
                        )
                    }
                    Layout::License => return window::resize(700, 450),
                    Layout::Help => unreachable!(),
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self.layout {
            // The default one column suggestion list layout
            Layout::Default => {
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
            // License text layout
            Layout::License => scrollable(
                column![include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/LICENSE"
                ))]
                .width(Length::Fill),
            )
            .into(),
            // Help menu
            Layout::Help => {
                todo!()
            }
        }
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

        for module in &mut self.modules {
            module.get_matches(trimmed_query, &mut self.suggestions)
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
                        container(sg.view())
                            .style(theme::Container::Custom(Box::new(
                                ContainerBackgroundStyle::new(Color::from([0.3, 0.3, 0.3])),
                            )))
                            .into()
                    } else {
                        sg.view()
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
