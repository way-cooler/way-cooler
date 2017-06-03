# ActiveContainerId

## Description
Returns the ID associated with the current active container.

## Return Value
`container_id` - The UUID as a UTF-8 encoded string.

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
print("The active container id is %s" % active_id)
```
