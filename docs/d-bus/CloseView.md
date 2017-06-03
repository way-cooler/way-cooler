# CloseView

## Description
Closes the `View` that is associated with the given UUID.

If no such view exists an error is returned.

If the UUID does not point to a `View` then an error is returned and no action is taken.

## Parameters
`view_id` - The UUID as a UTF-8 string of the view you want to remove.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.CloseView(active_id)
```
