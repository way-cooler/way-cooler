//! Types defined by Lua thread

use hlua;
use hlua::{Lua, LuaError};
use hlua::any::AnyLuaValue;

use rustc_serialize::json::Json;

use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;

/// Represents an identifier for dealing with nested tables.
///
/// To access foo.bar.baz, use vec!["foo", "bar", "baz"].
///
/// To access foo[2], use vec!["foo", 2].
pub type LuaIdent = Vec<String>;

/// Methods that the Lua thread can execute.
pub type LuaFunc = fn(&mut Lua);

/// Messages sent to the lua thread
pub enum LuaQuery {
    /// Pings the lua thread
    Ping,
    /// Halt the lua thread
    Terminate,
    // Restart the lua thread
    Restart,

    /// Execute a string
    Execute(String),
    /// Execute a file
    ExecFile(String),

    /// Get a variable, expecting an AnyLuaValue
    GetValue(LuaIdent),
    /// Invoke a function found at the position,
    /// with the specified arguments.
    Invoke(LuaIdent, Vec<AnyLuaValue>),
    /// Set a value
    SetValue(LuaIdent, Json),
    /// Create a new table
    NewTable(LuaIdent),
    /// Execute some Rust using the Lua context.
    ExecWithLua(LuaFunc),
}

impl Debug for LuaQuery {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &LuaQuery::Ping => write!(f, "LuaQuery::Ping"),
            &LuaQuery::Terminate => write!(f, "LuaQuery::Terminate"),
            &LuaQuery::Restart => write!(f, "LuaQuery::Restart"),
            &LuaQuery::Execute(ref val) =>
                write!(f, "LuaQuery::Execute({:?})", val),
            &LuaQuery::ExecFile(ref val) =>
                write!(f, "LuaQuery::ExecFile({:?})", val),
            &LuaQuery::GetValue(ref val) =>
                write!(f, "LuaQuery::GetValue({:?})", val),
            &LuaQuery::Invoke(ref ident, ref val) =>
                write!(f, "LuaQuery::Invoke({:?}, {:?})", ident, val),
            &LuaQuery::SetValue(ref name, ref json) =>
                write!(f, "LuaQuery::SetValue({:?}, {:?})", name, json),
            &LuaQuery::NewTable(ref name) =>
                write!(f, "LuaQuery::NewTable({:?})", name),
            // This is why there's no #[derive(Debug)],
            // and why we have lua/types.rs
            &LuaQuery::ExecWithLua(_) =>
                write!(f, "LuaQuery::ExecWithLua()")
        }
    }
}

unsafe impl Send for LuaQuery { }
unsafe impl Sync for LuaQuery { }

/// Messages received from lua thread
pub enum LuaResponse {
    /// If the identifier had length 0
    InvalidName,
    /// Lua variable obtained
    Variable(Option<AnyLuaValue>),
    /// Lua error
    Error(hlua::LuaError),
    /// A function is returned
    Function(hlua::functions_read::LuaFunction<String>),
    /// Pong response from lua ping
    Pong,
}

unsafe impl Send for LuaResponse { }
unsafe impl Sync for LuaResponse { }

impl Debug for LuaResponse {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            LuaResponse::InvalidName =>
                write!(f, "LuaReponse::InvalidName"),
            LuaResponse::Variable(ref var) =>
                write!(f, "LuaResponse::Variable({:?})", var),
            LuaResponse::Error(ref err) =>
                write!(f, "LuaResponse::Error({:?})", err),
            LuaResponse::Function(_) =>
                write!(f, "LuaResponse::Function"),
            LuaResponse::Pong =>
                write!(f, "LuaResponse::Pong")
        }
    }
}
