use iced::widget::{column, container, horizontal_rule, horizontal_space, row, text, text_input};
use iced::{
    alignment, executor, keyboard, subscription, theme, window, Application, Background, Color,
    Command, Element, Event, Length, Settings, Theme,
};

//struct LanchFlags {
//    cache: Cache,
//}

fn lanch_settings() -> iced::Settings<()> {
    let mut settings: iced::Settings<()> = Settings::default();

    settings.id = Some(String::from("lanch")); // this sets the WM_CLASS on x11 and makes it easy
                                               // to define the window as floating in tiling
                                               // window managers

    settings.window.decorations = false;
    settings.window.always_on_top = true;
    settings.window.position = window::Position::Centered; // Centered for now
                                                           // TODO: PR for iced to add more modes
    settings.window.size = (600, 400);

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
#[allow(unused)]
enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
#[allow(unused)]
enum LanchMessage {
    QueryChanged(String),
    NavigateList(Direction),
    ExecuteSelected,
    Escape,
    SuggestionMessage(SuggestionMessage),
}

struct Lanch {
    query: String,
    suggestions: Vec<Box<dyn Suggestion>>,
    selected: usize,
    theme: Theme,
}

impl Lanch {
    // generates suggestions based on the query
    fn generate_suggestions(&mut self) {
        
    }

    // takes a key event and returns a message
    fn handle_key(
        key_code: keyboard::KeyCode,
        modifiers: keyboard::Modifiers,
    ) -> Option<LanchMessage> {
        use keyboard::{KeyCode, Modifiers};

        match key_code {
            // I don't think it's possible to have these in one statement :/ (guard works for both
            // patterns with |, and we only want it for the J)
            KeyCode::Down => Some(LanchMessage::NavigateList(Direction::Down)),
            KeyCode::J if modifiers == Modifiers::CTRL => Some(LanchMessage::NavigateList(Direction::Down)),

            KeyCode::Up  => Some(LanchMessage::NavigateList(Direction::Up)),
            KeyCode::K if modifiers == Modifiers::CTRL => Some(LanchMessage::NavigateList(Direction::Up)),

            KeyCode::Enter => Some(LanchMessage::ExecuteSelected),
            KeyCode::Escape => Some(LanchMessage::Escape),
            _ => None,
        }
    }
}

impl Application for Lanch {
    type Message = LanchMessage;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Lanch, Command<Self::Message>) {
        (
            Lanch {
                theme: Theme::Dark,
                query: String::new(),
                //suggestions: Vec::new(),
                suggestions: vec![
                    Box::new(Program::with_name("firefox")),
                    Box::new(Program::with_name("st")),
                    Box::new(Program::with_name("Gimp")),
                    Box::new(Program::with_name("Emacs")),
                ],
                selected: 1,
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
                self.query = q;
                self.selected = 0;
                self.generate_suggestions();
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

        let suggestions = if self.suggestions.is_empty() {
            // No query matches or empty query, show adequate message
            let info = if self.query.is_empty() {
                "Enter a query to get suggestions."
            } else {
                "No matches."
            };

            container(
                text(info)
                    .style(Color::from([0.7, 0.7, 0.7]))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
        } else {
            let list = column(
                // Display all the suggestions
                self.suggestions
                    .iter()
                    .enumerate()
                    .map(|(i, sg)| {
                        if i == self.selected {
                            container(column![sg
                                .view()
                                .map(move |message| LanchMessage::SuggestionMessage(message))])
                            .style(theme::Container::Custom(Box::new(SelectedSuggestionStyle)))
                            .into()
                        } else {
                            sg.view()
                                .map(move |message| LanchMessage::SuggestionMessage(message))
                        }
                    })
                    .collect(),
            )
            .spacing(5);

            container(list)
        };

        container(column![
            input,
            horizontal_rule(3),
            row![
                horizontal_space(Length::FillPortion(1)),
                suggestions.width(Length::FillPortion(40)),
                horizontal_space(Length::FillPortion(1)),
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

mod suggestion;
use suggestion::program::*;
use suggestion::*;

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
