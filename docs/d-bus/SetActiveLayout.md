# SetActiveLayout

## Description
Sets the active container to the given layout.

## Parameters
`layout` - A UTF-8 encoded string representing one of the following layout styles:
* `"vertical"`
* `"horizontal"`
* `"tabbed"`
* `"stacked"`


## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
layout.SetActiveLayout("horizontal")
```
