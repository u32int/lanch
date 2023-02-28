use iced::widget::{
    column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input,
    vertical_space,
};
use iced::{
    alignment, executor, keyboard, subscription, theme, window, Application, Background, Color,
    Command, Element, Event, Length, Settings, Theme,
};

use std::collections::VecDeque;
use std::rc::Rc;

mod suggestion;
use suggestion::program::*;
use suggestion::*;

mod cache;
use cache::*;

#[derive(Default, Debug)]
struct LanchFlags {
    cache: LanchCache,
    options: LanchOptions,
}

#[derive(Default, Debug)]
struct LanchOptions {
    window_size: (u32, u32),
    font_size: f32,
}

fn lanch_settings() -> iced::Settings<LanchFlags> {
    let mut settings: iced::Settings<LanchFlags> = Settings {
        flags: LanchFlags {
            cache: LanchCache::from_disk_or_new().unwrap(),
            options: LanchOptions {
                window_size: (600, 400),
                font_size: 20f32,
            },
        },
        ..Default::default()
    };

    settings.id = Some(String::from("lanch")); // this sets the WM_CLASS on x11 and makes it easy
                                               // to define the window as floating in tiling
                                               // window managers

    settings.window.decorations = false;
    settings.window.always_on_top = true;
    settings.window.position = window::Position::Centered; // Centered for now
                                                           // TODO: PR for iced to add more modes

    settings.window.size = (
        settings.flags.options.window_size.0,
        settings.flags.options.font_size as u32 * 5,
    );
    settings.default_text_size = settings.flags.options.font_size;
    settings
}

fn main() -> iced::Result {
    match Lanch::run(lanch_settings()) {
        Ok(()) => (),
        Err(e) => return Err(e),
    }

    Ok(())
}

// Could possibly be extended for grid layouts
#[derive(Debug, Clone)]
enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
enum LanchMessage {
    QueryChanged(String),
    NavigateList(Direction),
    ExecuteSelected,
    Escape,
    SuggestionMessage(SuggestionMessage),
}

struct Lanch {
    // initialized from flags, contain the default values for window settings etc.
    options: LanchOptions,

    // cached programs to limit potentially hundreds of fs calls to 1 cache.
    // TODO: refresh periodically
    program_cache: Vec<Rc<dyn Suggestion>>,

    // loaded modules providing extra suggestion functionality
    modules: Vec<Rc<dyn Suggestion>>,

    // the current query in the main text box
    query: String,

    // suggestions to display to the user
    suggestions: Vec<Rc<dyn Suggestion>>,

    // indicates to the rendering code where to put section separators
    suggestion_separators: VecDeque<(usize, String)>,

    // the currently selected suggestion
    selected: usize,

    // application theme
    theme: Theme,
}

impl Lanch {
    // gernerates suggestions based on the query, updating the suggestions.
    fn generate_suggestions(&mut self) {
        self.suggestions.clear();
        self.suggestion_separators.clear();
        if self.query.is_empty() {
            return;
        }

        let mut match_counter: usize = 0;

        for module in &self.modules {
            if module.matches(&self.query) {
                self.suggestions.push(Rc::clone(module));
                self.suggestion_separators
                    .push_back((match_counter, module.to_string()));
                match_counter += 1;
            }
        }

        self.suggestion_separators
            .push_back((match_counter, String::from("Programs")));
        // filter and push matching program suggestions
        self.program_cache
            .iter()
            .filter(|program| program.matches(&self.query))
            .for_each(|program| self.suggestions.push(Rc::clone(program)));
    }

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

        let mut sep_counter = 0;

        //println!("RENDERING STRUCTURE\n{:#?}", self.suggestions);

        let list = column(
            // Display all the suggestions
            self.suggestions
                .iter()
                .enumerate()
                .map(|(i, sg)| {
                    let elem = if i == self.selected {
                        container(
                            sg.view()
                                .map(move |message| LanchMessage::SuggestionMessage(message)),
                        )
                        .style(theme::Container::Custom(Box::new(SelectedSuggestionStyle)))
                        .into()
                    } else {
                        sg.view()
                            .map(move |message| LanchMessage::SuggestionMessage(message))
                            .into()
                    };

                    if let Some(sep) = self.suggestion_separators.get(sep_counter) {
                        if sep.0 == i {
                            sep_counter += 1;
                            return column![
                                row![
                                    text(&sep.1),
                                    horizontal_space(5),
                                    column![
                                        vertical_space(self.options.font_size / 2f32),
                                        row![
                                            horizontal_rule(1),
                                            horizontal_space(self.options.font_size / 2f32)
                                        ]
                                    ]
                                ],
                                elem,
                            ]
                            .spacing(5)
                            .into();
                        }
                    }

                    elem
                })
                .collect(),
        )
        .spacing(5)
        .width(Length::Fill);

        scrollable(list).into()
    }

    // takes a key event and returns a message
    fn handle_key(
        key_code: keyboard::KeyCode,
        modifiers: keyboard::Modifiers,
    ) -> Option<LanchMessage> {
        use keyboard::{KeyCode, Modifiers};

        match key_code {
            // I don't think it's possible to have these in one statement :/ (guards work for both
            // patterns with |, and we only want it for the J)
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
                modules: vec![
                    Rc::new(timedate::TimeSuggestion),
                    Rc::new(help::HelpSuggestion),
                ], // temporary, will load from
                // config eventually
                query: String::new(),
                suggestions: Vec::new(),
                suggestion_separators: VecDeque::new(),
                selected: 1,
                theme: Theme::Dark,
            },
            window::gain_focus(),
        )
    }

    fn title(&self) -> String {
        String::from("launch")
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            LanchMessage::QueryChanged(q) => {
                self.query = q.trim().to_string();
                self.selected = 0;
                self.generate_suggestions();

                if self.suggestions.is_empty() {
                    return window::resize(
                        self.options.window_size.0,
                        self.options.font_size as u32 * 5,
                    );
                } else {
                    return window::resize(
                        self.options.window_size.0,
                        (self.options.font_size as u32
                            * (self.suggestions.len() + self.suggestion_separators.len() + 3)
                                as u32)
                            .clamp(0, self.options.window_size.1),
                    );
                }
            }
            LanchMessage::NavigateList(d) => match d {
                Direction::Up => {
                    self.selected = (self.selected.checked_sub(1).unwrap_or(0))
                        .clamp(0, self.suggestions.len());
                }
                Direction::Down => {
                    self.selected = (self.selected + 1).clamp(0, self.suggestions.len() - 1);
                }
            },
            LanchMessage::ExecuteSelected => {
                self.suggestions.get(self.selected).unwrap().execute();

                return window::close();
            }
            LanchMessage::Escape => {
                return window::close();
            }
            _ => todo!(),
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let input =
            text_input("Search...", &self.query, LanchMessage::QueryChanged).width(Length::Fill);

        let suggestions = container(self.view_suggestions());

        container(column![
            input,
            horizontal_rule(3),
            row![
                horizontal_space(Length::FillPortion(1)),
                suggestions.width(Length::FillPortion(40)),
            ]
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

struct SelectedSuggestionStyle;

impl container::StyleSheet for SelectedSuggestionStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from([0.3, 0.3, 0.3]))),
            text_color: Some(style.palette().text),
            ..Default::default()
        }
    }
}
