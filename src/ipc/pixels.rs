use dbus::arg::{Array};
use dbus::MessageItem;

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
                // Also can just return a (&*result).into() (a slice) which IS faster...
                //let result = vec![5u8];
                let result: Vec<MessageItem> = vec![MessageItem::UInt32(5u32)];
                Ok(vec![m.msg.method_return().append((MessageItem::Array(result, "(u)".into())))])
            }).outarg::<Array<u32, Vec<u32>>, _>("success")
        )
    )
}
