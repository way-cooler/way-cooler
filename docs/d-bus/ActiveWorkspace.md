# ActiveWorkspace

## Description
Gets the name of the current active workspace.

## Return Value
`name` - A UTF-8 encoded string representing the current workspace name.

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
current_workspace = layout.ActiveWorkspace()
print("The current workspace is %s" % current_workspace)
```
