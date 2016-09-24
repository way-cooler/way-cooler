/// The dbus module. Hide anything related to the dbus library here.

use dbus::tree::{self, Tree, Factory, Method, MethodInfo, MethodResult, MTFn};
use dbus::{Connection, BusType, NameFlag};

use layout::commands::{tile_switch, focus_left};

pub fn init() {
    let connection = Connection::get_private(BusType::Session)
        .expect("Unable to create dbus session");
    connection.register_name("org.way-cooler", NameFlag::AllowReplacement as u32)
        .expect("Unable to register 'org.way-cooler' on dbus");

    let factory = Factory::new_fn::<()>();

    let tree = factory.tree()
        .add(factory.object_path("/layout", ()).introspectable()
             .add(factory.interface("org.way-cooler.Layout", ())
                  .add_m(factory.method("SwitchActiveTileMode", (),
                                        move |m| {
                                            tile_switch();
                                            Ok(vec![])
                                        }))
                  .add_m(factory.method("FocusLeft", (),
                                        move |m| {
                                            focus_left();
                                            Ok(vec![])
                                        }))
                  ));
    tree.set_registered(&connection, true);

    for client in tree.run(&connection, connection.iter(1000)) {
        trace!("Got a commection: {:?}", client);
    }
}
