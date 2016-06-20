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

-- Register keybindings
config.register_keys = function(keys)
    keys = assert(keys, "key table was nil" .. use_key)
    keys = assert(type(keys) == 'table', "keys must be a table" .. use_key)
    for _,v in pairs(keys) do
        if type(v) ~= 'table' then
            error("invalid keybinding given" .. use_key, 2)
        end
        assert(v.mods, "keybinding missing modifiers" .. use_key)
        assert(v.key, "keybinding missing modifiers" .. use_key)
        assert(v.action, "keybinding missing action" .. use_key)
        assert(v.loop, "keybinding missing repeat" .. use_key)
        assert(type(v.mods) == 'table',
               "keybinding modifiers: expected table" .. use_key)
        assert(type(v.key) == 'string',
               "keybinding key: expected string" .. use_key)
        assert(type(v.action) == 'string' or type(v.action) == 'function',
               "keybinding action: expected string or table" .. use_key)
        assert(type(v.loop) == 'boolean',
               "keybinding repeat: expected optional boolean" .. use_key)
    end
end

return config
