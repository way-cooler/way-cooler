/// Generated modules from the XML protocol spec.
pub use self::generated::client;

mod generated {
    // Generated code generally doesn't follow standards
    #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
    #![allow(non_upper_case_globals,non_snake_case,unused_imports)]

    pub mod c_interfaces {
        use wayland_client::sys::common::*;
        use wayland_client::sys::protocol_interfaces::*;
        #[doc(hidden)]
        include!(concat!(env!("OUT_DIR"), "/xdg-shell_interface.rs"));
    }

    pub mod client {
        #[doc(hidden)]
        use wayland_client::*;
        use wayland_client::commons::*;
        #[doc(hidden)]
        use wayland_client::protocol::*;
        #[doc(hidden)]
        use super::c_interfaces;
        include!(concat!(env!("OUT_DIR"), "/xdg-shell_api.rs"));
    }
}
