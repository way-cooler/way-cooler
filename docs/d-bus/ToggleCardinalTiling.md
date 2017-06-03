# ToggleCardinalTiling

## Description
Toggles how a container will layout its children between `horizontal` and `vertical`.

If the container is neither `horizontal` or `vertical`, it will default to `horizontal`.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to change the layout of.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.ToggleCardinalTiling(active_id)
```
