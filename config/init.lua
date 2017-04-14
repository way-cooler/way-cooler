-- Lua configration file for way-cooler. Ran at startup and when restarted.

--
-- Background
--
--
-- A background can either be a 6 digit hex value or an image path
local background = 0x5E4055

-- Programs that Way Cooler can run
way_cooler.programs = {
  -- Name of the window that will be the bar window.
  -- This is a hack to get X11 bars and non-Way Cooler supported bars working.
  --
  -- Make sure you add the script to start your bar in the init function!
  x11_bar = "lemonbar"
}

-- Registering programs to run at startup
-- These programs are only ran once util.program.spawn_programs is called.
util.program.spawn_at_startup("way-cooler-bg", "--color " .. background)

-- These options are applied to all windows.
way_cooler.windows = {
  gaps = { -- Options for gaps
    size = 0, -- The width of gaps between windows in pixels
  },
  borders = { -- Options for borders
    size = 20, -- The width of the borders between windows in pixels
    inactive_color = 0x386890, -- Color of the borders for inactive containers
    active_color = 0x57beb9 -- Color of active container borders
  },
  title_bar = { -- Options for title bar above windows
    size = 20, -- Size of the title bar
    background_color = 0x386690, -- Color of inactive title bar
    active_background_color = 0x57beb9, -- Color of active title bar
    font_color = 0x0, -- Color of the font for an inactive title bar
    active_font_color = 0xffffff -- Color of font for active title bar
  }
}

-- Options that change how the mouse behaves.
way_cooler.mouse = {
  -- Locks the mouse to the corner of the window the user is resizing.
  lock_to_corner_on_resize = false
}

--
-- Keybindings
--
-- Create an array of keybindings and call way_cooler.register_keys()
-- to register them.
-- Declaring a keybinding:
-- key(<modifiers list>, <key>, <function or name>, [repeat])

-- <modifiers list>: Modifiers (mod4, shift, control) to be used

-- <key>: Name of the key to be pressed. See xkbcommon keysym names.

-- <function or name> If a string, the way-cooler command to be run.
-- If a function, a Lua function to run on the keypress. The function takes
-- a list of key names as input (i.e. { "mod4", "shift", "a" }) if needed.

-- [repeat]: Optional boolean defaults to true - if false, the command will
-- will not follow "hold down key to repeat" rules, and will only run once,
-- waiting until the keys are released to run again.

-- Modifier key used in keybindings. Mod3 = Alt, Mod4 = Super/Logo key
mod = "Alt"

-- Aliases to save on typing
local key = way_cooler.key

local keys = {
  -- Open dmenu
  key({ mod }, "d", util.program.spawn_once("dmenu_run")),
  -- Open terminal
  key({ mod }, "return", util.program.spawn_once("weston-terminal")),

  -- Lua methods can be bound as well
  key({ mod, "Shift" }, "h", function () print("Hello world!") end),

  -- Some Lua dmenu stuff
  key({ mod }, "l", "dmenu_eval"),
  key({ mod, "Shift" }, "l", "dmenu_lua_dofile"),

  -- Move focus
  key({ mod }, "left", "focus_left"),
  key({ mod }, "right", "focus_right"),
  key({ mod }, "up", "focus_up"),
  key({ mod }, "down", "focus_down"),

  -- Move active container
  key({ mod, "Shift" }, "left", "move_active_left"),
  key({ mod, "Shift" }, "right", "move_active_right"),
  key({ mod, "Shift" }, "up", "move_active_up"),
  key({ mod, "Shift" }, "down", "move_active_down"),

  -- Split containers
  key({ mod }, "h", "split_horizontal"),
  key({ mod }, "v", "split_vertical"),
  key({ mod }, "e", "horizontal_vertical_switch"),
  key({ mod }, "f", "fullscreen_toggle"),
  key({ mod, "Shift" }, "q", "close_window"),
  key({ mod, "Shift" }, "space", "toggle_float_active"),
  key({ mod }, "space", "toggle_float_focus"),
  key({ mod, "Shift" }, "r", "way_cooler_restart"),

  -- Quitting way-cooler is hardcoded to Alt+Shift+Esc.
  -- If rebound, then this keybinding is cleared.
  --key({ mod, "Shift" }, "escape", "way_cooler_quit"),
}

-- Add Mod + X bindings to switch to workspace X, Mod+Shift+X send active to X
for i = 1, 9 do
  table.insert(keys,
               key({ mod }, tostring(i), "switch_workspace_" .. i))
  table.insert(keys,
               key({ mod, "Shift" }, tostring(i), "move_to_workspace_" .. i))
end

-- Register the keybindings.
for _, key in pairs(keys) do
    way_cooler.register_key(key)
end

-- Register the mod key to also be the mod key for mouse commands
way_cooler.register_mouse_modifier(mod)

-- Execute some code after Way Cooler is finished initializing
way_cooler.on_init = function()
  util.program.spawn_startup_programs()
end

--- Execute some code when Way Cooler restarts
way_cooler.on_restart = function()
  util.program.restart_startup_programs()
end

--- Execute some code when Way Cooler terminates
way_cooler.on_terminate = function()
  util.program.terminate_startup_programs()
end
