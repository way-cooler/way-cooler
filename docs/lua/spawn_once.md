# spawn_once

## Description
Returns a function that spawns a program once. The function takes no arguments and returns no value.

Does not update the global program spawn list.

Used primarily for key mapping 

## Parameters
`bin` - The program to run. Can be an absolute path or a command to run.
`args` - The arguments (as a string) to pass to the program.

## Return Value
`callback` - The callback that, when triggered, will spawn the program.

## Examples
```lua
-- Spawns a terminal when you press <mod>+s
spawn_key = key({ mod }, "s", util.program.spawn("weston-terminal"))
way_cooler.register_key(spawn_key)
```

```lua
-- Convoluted way to spawn a program at startup
func = util.program.spawn_once("weston-terminal")
func()

```
