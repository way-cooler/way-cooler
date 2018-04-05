//! Types defined by Lua thread

use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;

use rlua;

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
    /// Execute some Rust using the Lua context.
    ExecRust(fn(&rlua::Lua) -> rlua::Value<'static>),
    /// Execute some Rust using the Lua context.
    ExecWithLua(Box<FnMut(&rlua::Lua) -> rlua::Result<()>>)
}

/// Messages received from lua thread
pub enum LuaResponse {
    /// If the identifier had length 0
    InvalidName,
    /// Lua variable obtained
    Variable(Option<rlua::Value<'static>>),
    /// Lua error
    Error(rlua::Error),
    /// A function is returned
    Function(rlua::Function<'static>),
    /// Pong response from lua ping
    Pong
}

impl LuaResponse {
    /// Whether this response is an InvalidName or Error
    pub fn is_err(&self) -> bool {
        match self {
            &LuaResponse::InvalidName | &LuaResponse::Error(_) => true,
            _ => false
        }
    }

    /// If this response is a Variable, Function, or Pong
    #[inline]
    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

impl Debug for LuaResponse {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            LuaResponse::InvalidName => write!(f, "LuaReponse::InvalidName"),
            LuaResponse::Variable(ref var) => write!(f, "LuaResponse::Variable({:?})", var),
            LuaResponse::Error(ref err) => write!(f, "LuaResponse::Error({:?})", err),
            LuaResponse::Function(_) => write!(f, "LuaResponse::Function"),
            LuaResponse::Pong => write!(f, "LuaResponse::Pong")
        }
    }
}

impl Debug for LuaQuery {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            LuaQuery::Ping => write!(f, "LuaQuery::Ping"),
            LuaQuery::Terminate => write!(f, "LuaQuery::Terminate"),
            LuaQuery::Restart => write!(f, "LuaQuery::Restart"),
            LuaQuery::Execute(ref val) => write!(f, "LuaQuery::Execute({:?})", val),
            LuaQuery::ExecFile(ref val) => write!(f, "LuaQuery::ExecFile({:?})", val),
            LuaQuery::ExecRust(_) => write!(f, "LuaQuery::ExecRust()"),
            LuaQuery::ExecWithLua(_) => write!(f, "LuaQuery::ExecWithLua()")
        }
    }
}
