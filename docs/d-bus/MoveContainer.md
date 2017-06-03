# MoveContainer

## Description
Moves a tiled container in the desired direction. This only works on non-floating containers.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to move.

`direction` - A UTF-8 encoded string representing one of the four cardinal directions:
* `"up"`
* `"down"`
* `"left"`
* `"right"`

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.MoveContainer(active_id, "right")
```
