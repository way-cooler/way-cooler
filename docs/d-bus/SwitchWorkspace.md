# SwitchWorkspace

## Description
Switches to a named workspace. If the named workspace is on a different output, both the focused workspace and the focused output is changed.

If the workspace does not exist yet, a new one is created on the currently focused output.

## Parameters
`w_name` - A UTF-8 encoded string representing the name of the workspace to switch to.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
first_active_id = layout.ActiveContainerId()

# Note that this can be ANY string, 
# including one not specified in the configuration file
layout.SwitchWorkspace("a different workspace")
second_active_id = layout.ActiveContainerId()
assert(first_active_id != second_active_id)
```

