use wayland_client::{GlobalError, GlobalManager, Interface, NewProxy, Proxy};

pub fn instantiate_global<F, I>(
    globals: &GlobalManager,
    version: u32,
    implementor: F,
    name: &str
) where
    I: Interface + From<Proxy<I>>,
    F: FnOnce(NewProxy<I>) -> I
{
    globals
        .instantiate_exact(version, implementor)
        .unwrap_or_else(|err| match err {
            GlobalError::Missing => {
                error!("Missing {} protocol (need version {})", name, version);
                crate::fail(&format!(
                    "Your compositor doesn't support the {} protocol. \
                     This protocol is necessary for Awesome to function",
                    name
                ));
            },
            GlobalError::VersionTooLow(actual_version) => {
                error!(
                    "Got version {} of the {} protocol, expected version {}",
                    actual_version, name, version
                );
                crate::fail(&format!(
                    "Your compositor doesn't support version {} \
                     of the {} protocol. Ensure your compositor is up to date",
                    name, version
                ));
            }
        });
}
