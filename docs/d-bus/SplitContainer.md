# SplitContainer

## Description
Changes how a container will layout its children.

If the container is a `View`, it creates a new sub-container with the only child being that view.

If the container is a `Container`, then that container's tile layout changes to the provided layout.


## Parameters
`container_id` - The UUID as a UTF-8 string of the container you want to change the layout of.

`split_axis` - A UTF-8 encoded string representing one of the following layout styles:
* `"vertical"`
* `"horizontal"`
* `"tabbed"`
* `"stacked"`

## Return Value
`success` - Unused boolean value, always returns True

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
active_id = layout.ActiveContainerId()
layout.SplitContainer(active_id, "vertical")
```
