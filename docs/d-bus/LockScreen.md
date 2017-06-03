# LockScreen

## Description
Locks Way Cooler and spawns the special lock screen program.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
layout.LockScreen()
```
