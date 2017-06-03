# ContainerInActiveWorkspace

## Description
Determines whether or not the container associated with the UUID is in the active workspace or not.

If the container is on a workspace that is on another output that is not focused, this method will return `false`. The only criteria for active workspace is if the user is focused on a container in that workspace.

*Do not rely on this method to tell if a container is visible to the user or not*.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you think is in the active workspace.

## Return Value
`success` - Boolean value that is `true` if the container is in the active workspace, `false` otherwise.

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
is_in_active = layout.ContainerInActiveWorkspace(active_id)
assert(is_in_active)
```
