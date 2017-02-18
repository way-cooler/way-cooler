-- Private table of Rust functions

-- Our connection to Rust functions exposed by Way Cooler
local rust = __rust
__rust = nil

-- The table that the user sees
way_cooler = {}
-- The meta table magic that way_cooler uses to talk to Way Cooler
local way_cooler_mt = {}
-- The commands that way_cooler can run, e.g way_cooler.key(...)
local commands = {}
-- The key mapping that is updated by way_cooler.key(...)
__key_map = {}
-- A cache of the registry, this is used to push values to Way Cooler.
-- Values are pushed here, and then we inform Way Cooler to read them.
__registry_cache = {}
local registry_cache_mt = {}

-- Initialize the workspaces
commands.init_workspaces = function(settings)
    assert(type(settings) == 'table', "settings: expected table")
    for ix, val in pairs(settings) do
        assert(type(ix) == 'number', "settings: expected number-indexed array")
        assert(type(val) == 'table', "settings: expected array of tables")
        val.name = val.name or ""
        val.mode = val.mode or "tiling"
    end
    rust.init_workspaces(settings)
end

-- Create a new keybinding to register with Rust
commands.key = function(mods, key, action, loop)
    assert(type(mods) == 'table', "modifiers: expected table")
    assert(type(key) == 'string', "key: expected string")
    if loop == nil then loop = true end
    if type(action) ~= 'string' and type(action) ~= 'function' then
        error("action: expected string or function", 2)
    end
    return {
        mods = mods, key = key, action = action, loop = loop
    }
end

commands.on_init = function() end
commands.on_restart = function() end
commands.on_terminate = function() end

local use_key = ", use the `key` or `way_cooler.key` method to create a keybinding"
-- Converts a list of modifiers to a string
local function keymods_to_string(mods, key)
    table.insert(mods, key)
    return table.concat(mods, ',')
end
-- Save the action at the __key_map and tell Rust to register the Lua key
local function register_lua_key(index, action, loop)
    local map_ix = rust.register_lua_key(index, loop)
    __key_map[map_ix] = action
end

-- Register a keybinding
commands.register_key = function(key)
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
        rust.register_command_key(keymods_to_string(key.mods, key.key),
                                  key.action, key.loop)
    elseif (type(key.action) == 'function') then
        register_lua_key(keymods_to_string(key.mods, key.key),
                              key.action, key.loop)
    else
        error("keybinding action: expected string or a function"..use_key, 2)
    end
end

-- Bind a key to use in conjunction with the mouse for certain commands (resize, move floating)
commands.register_mouse_modifier = function(mod)
  assert(type(mod) == 'string', "mod: expected a string")
  rust.register_mouse_modifier(mod)
end

way_cooler_mt.__index = function(_table, key)
    if commands[key] then
      return commands[key]
    end
    if type(key) ~= 'string' then
        error("Invalid key, string expected", 1)
    else
        if __registry_cache[key] then
          return __registry_cache[key]
        end
        return rust.ipc_get(key)
    end
end
way_cooler_mt.__newindex = function(_table, key, value)
    if type(value) == "function" then
      commands[key] = value
      return
    end
    if type(key) ~= 'string' then
        error("Invlaid key, string expected", 1)
    else
        __registry_cache[key] = value
        -- now read those values we just wrote to registry_cache.
        rust.ipc_set(key)
    end
end


way_cooler_mt.__to_string = function(_table)
    return "Way Cooler IPC access"
end

way_cooler_mt.__metatable = "Cannot modify"
commands.__metatable = "Cannot modify"

setmetatable(way_cooler, way_cooler_mt)
setmetatable(__key_map, { __metatable = "cannot modify" })
