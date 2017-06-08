# ToggleFloatingFocus

## Description
Toggles focusing between the floating view and the tiled containers.


## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.ToggleFloatingFocus()
layout.ToggleFloatingFocus()
assert!(active_id == layout.ActiveContainerId())
```
