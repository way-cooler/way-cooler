# SetPointerPos

## Description
Sets the absolute position of the pointer to the given coordinates.

If the values are beyond the range of the output, then they are clamped to the edge of the screen.

E.g on a 800x600 screen, an input of `x = 700` and `y = 700` will result in the pointer being set at `700x600`.

## Parameters
`x` - The x value of the coordinates, as a signed 32 bit number.

`y` - The y value of the coordinates, as a signed 32 bit number.

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
layout.SetPointerPos(100, 100)
```
