# FullScreen

## Description
Sets a container to be fullscreen / not fullscreen.

A fullscreen container is no longer tiled according to its parent container, and use the entire screen.

If a container is toggled to be in a state it is already in then no error is returned and no action is taken.

## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to move.

`toggle` - Boolean value. If `true`, the container is made fullscreen. If `false` the container is set to no longer be fullscreen.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.FullScreen(active_id)
```
