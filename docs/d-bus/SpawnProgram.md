# SpawnProgram

## Description
Spawns the program, on success returns the pid of the process

## Parameters
`prog_name` - The path to the program as a UTF-8 encoded string.

## Return Value
`pid` - The pid of the spawned process

## Examples
```python
from pydbus import SessionBus
bus = SessionBus()
layout = bus.get(bus_name='org.way-cooler', object_path='/org/way_cooler/Layout')
pid = layout.SpawnProgram("weston-terminal")
```
