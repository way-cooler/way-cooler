-- Lua configuration file for way-cooler.

local utils = require("utils") -- Utilities, i.e. shell

--
-- Layouts
--

-- The default layout options are no names, mode = "default" (use keybindings).
-- For a list of tiling options, see way-cooler docs or `man way-cooler-tiling`.
-- Workspaces, like arrays in Lua, start with 1.
local workspace_settings = {
  -- The first workspace is named web
  [1] = { name = "web" },
  -- The 9th workspace is named "free", and all windows sent there float.
  [9] = { name = "free", mode = "float" },
}

-- Create 9 workspaces with the given settings.
config.init_workspaces(9, workspace_settings)

--
-- Background
--

--way_cooler.background.path = "path/to/standard/background"
-- <options for folder, cycle, etc.>

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
mod = "Mod4"
local key = config.key -- Alias key so it's faster to type

way_cooler.terminal = "weston-terminal" -- Use the terminal of your choice

local keys = {
  -- Open dmenu
  key({ mod }, "d", "launch_dmenu"),
  -- Open terminal
  key({ mod }, "Enter", function()
      util.exec.spawn(terminal)
  end),

  -- Switch workspaces L/R/previous
  key({ mod }, "Left",      "workspace.switch_left"),
  key({ mod }, "Right",     "workspace.switch_right"),
  key({ mod }, "Backspace", "workspace.switch_previous"),

  -- Send active to workspace L/R
  key({ mod, "Shift" }, "Left",      "workspace.send_active_left", false),
  key({ mod, "Shift" }, "Right",     "workspace.send_active_right", false),
  key({ mod, "Shift" }, "Backspace", "workspace.send_active_previous", false),

  -- Move focus
  key({ mod }, "j", "layout.focus_left"),
  key({ mod }, "k", "layout.focus_right"),

  -- Quit
  key({ mod, "Shift" }, "q", "quit"),
}

-- Add Mod + X bindings to switch to workspace X, Mod+Shift+X send active to X
for i = 1, max_workspaces do
  util.table.append_to(keys,
    key({ mod          }, tostring(i), "workspace.switch_to" .. i),
    key({ mod, "Shift" }, tostring(i), "workspace.send_active_to_" .. i)
  )
end

-- Register the keybindings.
for _, key in pairs(keys) do
    config.register_keybinding(key)
end

-- To use plugins such as bars, or to start other programs on startup,
-- call util.exec.spawn_once, which will not spawn copies after a config reload.

-- util.exec.spawn_once("way-cooler-bar")

-- To add your own Lua files:
-- require("my-config.lua") -- Or use utils.hostname

-- !! Do not place any code after this comment.
-- !! way-cooler and plugins may insert auto-generated code.
