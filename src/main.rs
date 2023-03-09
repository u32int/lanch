mod cache;
mod suggestion;
mod ui;

use cache::*;

fn main() -> Result<(), iced::Error> {
    let cache = match LanchCache::from_disk_or_new(None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Error] Couldn't load cache: {}", e);
            std::process::exit(1);
        }
    };

    ui::init(cache)
}
