use dbus::arg::{Array};
use dbus::tree::MethodErr;
use dbus::MessageItem;

use ::ipc::utils::{parse_uuid, lock_tree_dbus};
use ::ipc::{DBusFactory, DBusObjPath};
use ::render::screen_scrape::{write_screen_scrape_lock, read_screen_scrape_lock,
                              scraped_pixels_lock, sync_scrape};

pub fn setup(f: &mut DBusFactory) -> DBusObjPath {
    f.object_path("/org/way_cooler/Screen", ()).introspectable().add(
        f.interface("org.way_cooler.Screen", ())
            .add_m(
                f.method("List", (), |m| {
                    let tree = lock_tree_dbus()?;
                    let outputs = tree.outputs().iter()
                        .map(|id|
                             MessageItem::Str(format!("{}", id.simple())))
                        .collect();
                    Ok(vec![m.msg.method_return()
                            .append((MessageItem::Array(outputs, "s".into())))
                    ])
                }).outarg::<Array<String, Vec<String>>, _>("success")
            )
            .add_m(
                // TODO Make this take in a:
                // * Output UUID
                // * Size (u32, u32) to scrape. For invalid ones, we throw error
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
            .add_m(
                f.method("Resolution", (), |m| {
                    let mut args_iter = m.msg.iter_init();
                    let output_id = args_iter.read::<String>()?;
                    let tree = lock_tree_dbus()?;
                    let uuid = parse_uuid("container_id", &output_id)?
                        .or_else(|| tree.active_id())
                        .ok_or(MethodErr::failed(&"No active container"))?;
                    let size = tree.output_resolution(uuid)
                        .map_err(|err| {
                            MethodErr::failed(&format!("{:?}", err))
                        })?;
                    Ok(vec![m.msg.method_return()
                            .append1((size.w, size.h))
                    ])
                })
                    .outarg::<(u32, u32), _>("success")
                    .inarg::<String, _>("output_id")
            )
    )
}
