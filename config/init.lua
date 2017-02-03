-- Lua configuration file for way-cooler. Ran at startup and when restarted.

-- TODO Move somewhere, or just remove
--
-- Background
--
--
-- A background can either be a 6 digit hex value or an image path
local background = 0x5E4055

--
-- Keybindings
--
-- Create an array of keybindings and call config.register_keys()
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
local key = config.key
local windows = config.windows

way_cooler.terminal = "weston-terminal" -- Use the terminal of your choice

-- Name of the window that will be the bar window.
-- This is a hack to get X11 bars and non-Way Cooler supported bars working.
--
-- Make sure you add the script to start your bar in the init function!
way_cooler.bar = "lemonbar"


-- These options are applied to all windows.
windows.all = {
  gaps = { -- Options for gaps
    size = 0, -- The width of gaps between windows in pixels
  },
  borders = { -- Options for borders
    size = 20, -- The width of the borders between windows in pixels
    inactive_color = 0x386890, -- Color of the borders for inactive containers
    active_color = 0x57beb9 -- Color of active container borders
  }
}

local keys = {
  -- Open dmenu
  key({ mod }, "d", "launch_dmenu"),
  -- Open terminal
  key({ mod }, "return", "launch_terminal"),

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
  key({ mod, "Shift" }, "r", "way_cooler_restart")

  -- Quitting way-cooler is hardcoded to Alt+Shift+Esc.
  -- If rebound, then clears the keybinding for Alt+Shift+Esc
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
    config.register_key(key)
end

-- Register the mod key to also be the mod key for mouse commands
config.register_mouse_modifier(mod)

function cleanup_background()
  os.execute("pkill way-cooler-bg")
end


-- Execute some code after Way Cooler is finished initializing
function way_cooler_init()
  local status = os.execute("which way-cooler-bg 2>/dev/null")
  if not status then
    print "Could not find way-cooler-bg! Please install it"
  else
    os.execute("way-cooler-bg " .. background .. " &")
  end
end

-- TODO can I just set these inline?
--- Execute some code when Way Cooler restarts
function way_cooler_restart()
  cleanup_background()
end

function way_cooler_terminate()
  cleanup_background()
end


way_cooler.on_restart(way_cooler_restart)
way_cooler.on_terminate(way_cooler_terminate)

-- To use plugins such as bars, or to start other programs on startup,
-- call util.exec.spawn_once, which will not spawn copies after a config reload.

-- util.exec.spawn_once("way-cooler-bar")

-- To add your own Lua files:
-- require("my-config.lua") -- Or use utils.hostname
