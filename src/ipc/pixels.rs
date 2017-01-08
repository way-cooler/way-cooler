use dbus::arg::{Array};
use dbus::MessageItem;

use rustwlc::render::{read_pixels, wlc_pixel_format};
use rustwlc::{Geometry, Point, Size};

use super::{DBusFactory, DBusObjPath};

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
                let geo = Geometry {
                    origin: Point { x: 0, y: 0 },
                    size: Size { w: 100, h: 100}
                };
                let result = read_pixels(wlc_pixel_format::WLC_RGBA8888, geo).1.iter_mut()
                    .map(|pixel| MessageItem::Byte(*pixel)).collect();
                Ok(vec![m.msg.method_return().append((MessageItem::Array(result, "y".into())))])
            }).outarg::<Array<u8, Vec<u8>>, _>("success")
        )
    )
}
