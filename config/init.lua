-- Lua configuration file for way-cooler.
local way_cooler = require("way_cooler") -- way-cooler IPC
local utils = require("utils") -- Utilities, i.e. shell
-- another library for lua way-cooler API? Things not in IPC?
--
-- Layouts
--

-- The default layout options are no names, mode = "default" (use keybindings).
-- For a list of tiling options, see way-cooler docs or `man way-cooler-tiling`.
-- Workspaces, like arrays in Lua, start with 1.
local workspace_settings = {
  -- The 9th workspace is named "free", and all windows sent there float.
  [9] = { name = "free", mode = "float" }
}

-- Apply the settings. For a list of functions, see `man way-cooler-lua`
-- Should this be a method in another API?
-- Some Lua functionality isn't in `way_cooler`
set_up_workspaces(workspace_settings)

-- This should be specified in a bar?
-- way_cooler.layout.clear_extra_layouts = false

--
-- Background
--

way_cooler.default_background = "path/to/standard/background"
-- <options for folder, cycle, etc.>

--
-- Keybindings
--

-- Modifier key used in keybindings. Mod3 = Alt, Mod4 = Super/Logo key
mod = "Mod4"

-- Create an array of keybindings and call register_keys() to register them.
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

local keys = util.table.add_all({ },
  -- Switch workspaces L/R/previous
  key({ mod }, "Left",      "workspace.switch_left"),
  key({ mod }, "Right",     "workspace.switch_right"),
  key({ mod }, "Backspace", "workspace.switch_previous"),

  -- Send active to workspace L/R
  key({ mod, "Shift" }, "Left",      "workspace.send_active_left", false),
  key({ mod, "Shift" }, "Right",     "workspace.send_active_right", false),
  key({ mod, "Shift" }, "Backspace", "workspace.send_active_to_previous", false)
)
-- Add Mod + X bindings to switch to workspace X, Mod+Shift+X send active to X
for i = 1, max_workspaces do
  keys = util.table.add_all(keys,
    key({ mod }, tostring(i), "workspace_switch_to" .. i),
    key({ mod, "Shift" }, tostring(i), "workspace.send_active_to_" .. i)
  )
end
