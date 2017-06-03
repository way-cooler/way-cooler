# FocusDir

## Description
Focuses on the container relative to the active container. The direction is based on the given relative direction.

## Parameters
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
layout.FocusDir("right")
```
