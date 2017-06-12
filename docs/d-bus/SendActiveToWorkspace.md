# SendActiveToWorkspace

## Description
Sends the active container to the named workspace.

If the workspace does not yet exist, a new one is created on the active output and the container is then sent there.

Note that this does **not** switch to the workspace, but focus will shift to the next available container in the current workspace.

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
layout.SendToWorkspace("a different workspace")
second_active_id = layout.ActiveContainerId()
assert(first_active_id != second_active_id)
```
