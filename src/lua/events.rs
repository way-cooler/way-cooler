//! Event API for the lua thread

// TODO look up Awesome API stuff
pub enum Event {
    Timer(String),
    /// Named channels that can send arbitrary data
    Message(String, AnyLuaValue),
    ViewCreated,
    ViewFocus,
}
