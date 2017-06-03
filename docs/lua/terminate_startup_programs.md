# terminate_startup_programs

## Description
Stops the startup programs. Does not remove them from the global list.

This means another call to `spawn_startup_programs` will respawn the programs killed by this function.
