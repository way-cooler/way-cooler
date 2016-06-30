-- Contains Lua glue for initializing things in the config

local config = {}

-- Private table of Rust functions
local rust = nil

-- Public method to set up private Rust data.
-- This is called in the lib init, and this method
-- and externally visible Rust tables are destroyed.
config.set_rust = function(interop, key)
    rust = interop
    return rust
end

-- Initialize the workspaces
config.init_workspaces = function(count, settings)
end

-- Create a new keybinding to register with Rust
config.key = function(mods, key, action, loop)
    mods = assert(type(mods) == 'table', "modifiers: expected table")
    key = assert(type(key) == 'string', "key: expected string")
    if loop == nil then loop = true end
    if type(action) ~= 'string' and type(action) ~= 'function' then
        error("action: expected string or function", 2)
    end
    return {
        mods = mods, key = key, action = action, loop = loop
    }
end

local use_key = ", use the `key` or `config.key` method to create a keybinding"

-- Register a keybinding
config.register_key = function(key)
    assert(key.mods, "keybinding missing modifiers" .. use_key)
    assert(key.key, "keybinding missing modifiers" .. use_key)
    assert(key.action, "keybinding missing action" .. use_key)
    assert(key.loop, "keybinding missing repeat" .. use_key)
    assert(type(key.mods) == 'table',
           "keybinding modifiers: expected table" .. use_key)
    assert(type(key.key) == 'string',
           "keybinding key: expected string" .. use_key)
    assert(type(key.loop) == 'boolean',
           "keybinding repeat: expected optional boolean" .. use_key)

    if (type(key.action) == 'string') then
        rust.register_command_key(key.mods, key.key, key.action, key.loop)
    elseif (type(key.action) == 'function') then
        rust.register_lua_key(key.mods, key.key, key.action, key.loop)
    else
        error("keybinding action: expected string or a function"..use_key, 2)
    end
end

return config
