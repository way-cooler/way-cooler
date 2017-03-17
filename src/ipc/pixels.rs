use dbus::arg::{Array};
use dbus::MessageItem;

use super::{DBusFactory, DBusObjPath};
use ::render::screen_scrape::{write_screen_scrape_lock, read_screen_scrape_lock,
                              scraped_pixels_lock, sync_scrape};

pub fn setup(f: &mut DBusFactory) -> DBusObjPath{
    //let f = Factory::new_fn::<()>();

    // TODO
    // Not working, choking on "1 lifetime parameter" expected for arg to outarg
    // Expanded it myself, seems fine, will have to debug later. For now writing
    // out the boiler plate is not that bad.
    /*
    dbus_interface! {
        path: "/org/way_cooler/pixels";
        name: "org.way_cooler.pixels";

        fn Get() -> success: DBusResult<Array<u32, Vec<u32>>> {
            let result: Vec<MessageItem> = vec![MessageItem::UInt32(5u32)];
            Ok(MessageItem::Array(result, "(u)".into()))
        }
    }
*/

    f.object_path("/org/way_cooler/Pixels", ()).introspectable().add(
        f.interface("org.way_cooler.Pixels", ()).add_m(
            f.method("hello", (), |m| {
                *write_screen_scrape_lock() = true;
                // ensure that no other threads can try to grab the pixels.
                let _lock = read_screen_scrape_lock();
                sync_scrape();
                trace!("IPC: pixel lock synchronized");
                drop(_lock);
                *write_screen_scrape_lock() = false;
                let scraped_pixels = scraped_pixels_lock()
                    .expect("scraped_pixels lock was poisoned!");
                let result = scraped_pixels.iter()
                    .map(|pixel| MessageItem::Byte(*pixel)).collect();
                Ok(vec![m.msg.method_return()
                        .append((MessageItem::Array(result, "y".into())))])
                /*let geo = Geometry {
                    origin: Point { x: 0, y: 0 },
                    size: Size { w: 100, h: 100}
                };
                let result = read_pixels(wlc_pixel_format::WLC_RGBA8888, geo).1.iter_mut()
                    .map(|pixel| MessageItem::Byte(*pixel)).collect();
                Ok(vec![m.msg.method_return().append((MessageItem::Array(result, "y".into())))])*/
            }).outarg::<Array<u8, Vec<u8>>, _>("success")
        )
    )
}
