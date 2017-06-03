# Focus

## Description
Focuses on the container associated with the given UUID.

If the container is already focused then no action will be taken and **no** error will be returned.

If the container cannot be focused on then no action will be taken and an error will be returned.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to focus on.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.Focus(active_id)
new_active_id = layout.ActiveContainerId()
assert(active_id == new_active_id)
```
