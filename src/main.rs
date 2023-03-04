mod cache;
mod suggestion;
mod ui;

use cache::*;

fn main() -> Result<(), iced::Error> {
    let cache = match LanchCache::from_disk_or_new() {
        Ok(c) => c,
        Err(e) => {
            println!("[Error] Couldn't load cache: {}", e);
            std::process::exit(1);
        }
    };

    ui::init(cache)
}
