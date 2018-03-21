# spawn_at_startup

## Description
Registers the program to spawn at startup and every time Way Cooler restarts.

## Parameters
`bin` - The program to run. Can be an absolute path or a command to run.
`args` - The arguments (as a string) to pass to the program.

## Examples
```lua
# Spawns the standard background program using an all white background.
# Every time Way Cooler is restarted, this program is killed and re-executed.
util.program.spawn_at_startup("way-cooler-bg", "--color " .. 0xFFFFFF)
```
