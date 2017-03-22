use dbus::arg::{Array};
use dbus::MessageItem;

use super::{DBusFactory, DBusObjPath};
use ::render::screen_scrape::{write_screen_scrape_lock, read_screen_scrape_lock,
                              scraped_pixels_lock, sync_scrape};

pub fn setup(f: &mut DBusFactory) -> DBusObjPath {
    f.object_path("/org/way_cooler/Screen", ()).introspectable().add(
        f.interface("org.way_cooler.Screen", ()).add_m(
            f.method("Scrape", (), |m| {
                *write_screen_scrape_lock() = true;
                // ensure that no other threads can try to grab the pixels.
                let _lock = read_screen_scrape_lock();
                sync_scrape();
                drop(_lock);
                *write_screen_scrape_lock() = false;
                let scraped_pixels = scraped_pixels_lock()
                    .expect("scraped_pixels lock was poisoned!");
                let result = scraped_pixels.iter()
                    .map(|pixel| MessageItem::Byte(*pixel)).collect();
                Ok(vec![m.msg.method_return()
                        .append((MessageItem::Array(result, "y".into())))])
            }).outarg::<Array<u8, Vec<u8>>, _>("success")
        )
    )
}
