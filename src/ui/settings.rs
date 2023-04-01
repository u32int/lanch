use iced::{window, Settings};

#[derive(Default, Debug)]
pub struct LanchFlags {
    pub options: LanchOptions,
}

#[derive(Default, Debug)]
pub struct LanchOptions {
    pub window_size: (u32, u32),
    pub font_size: f32,
}

pub fn settings() -> iced::Settings<LanchFlags> {
    let mut settings: iced::Settings<LanchFlags> = Settings {
        flags: LanchFlags {
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
