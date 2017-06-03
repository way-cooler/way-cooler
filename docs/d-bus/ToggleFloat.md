# ToggleFloat

## Description
Toggles a floating container to be tiled, or a tiled container to be floating.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to float/ground.

## Return Value
`success` - Unused boolean value, always returns True.

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.ToggleFloat(active_id)
```
