# spawn

## Description
Spawns the programs as if it was typed into the shell.

Does not update the global program spawn list.

## Parameters
`bin` - The program to run. Can be an absolute path or a command to run.
`args` - The arguments (as a string) to pass to the program.

## Examples
```lua
-- Spawns a terminal on startup
util.program.spawn("weston-terminal")
```

