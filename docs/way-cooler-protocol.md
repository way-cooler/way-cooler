# Structure
IPC with way-cooler consists of messages sent over a socket. This socket is
typically located at `/tmp/way-cooler/socket`. 

After connecting to this socket, two types of channels may be created: interaction
and event listening.

Communication in both pipes done by exchanging a series of "packets" back and forth
between the client and server.

# Packets

A packet will consist of a length followed by JSON payload.
The length shall be an unsigned 32-bit number (uint32) encoded in big-endian.
Following this will be a UTF-8 encoded string representing a valid JSON object.

Replies from the server will follow the same protocol.

# Basic communication

# Greeting Packet

Upon connecting to the server, a client must send a greeting packet which includes a
unique identifier, and specifies whether the client wants to send commands or listen
for events.

## Identifier

The identifier may be any string encoded in the `id` field of the request. If another
client has already registered with that ID, an error reply with a reason of 
`id taken` will be issued. If the ID is not a JSON string the error `invalid id` will be returned.

## Purpose

The `purpose` field shall be a string consisting of either `control` or `event`. This
determines the nature of the replies to the client.
If another client identified with the same name requesting events an error shall be
issued, `already registered`.
## Examples

```json
{ "id": "my-client", "purpose": "control" }

{ "id": "someon's client", "purpose": "event" }
```

# Communication

## Terminate
Sending a `quit` message will close the connection.
```json
{ "type": "quit" }
```

A `reason` can also be provided: 
```json
{ "type": "quit", "reason": "Got bored" }
```

At the moment, nothing will be done with the reason. In the future it may be emitted as an event.

## registry/get
Gets data from the registry.
```json
{ "type": "registry/get", "key": "foo" }
```

### Reply
The reply will either be a `value` with the requested value, or an error.
```json
{ "type": "success", "value": 12 }
```

### Errors
Errors will be returned if the registry key does not exist, or the registry
key cannot be accessed (if it is not an object or if it has restricted flags),
or if the value is write-only.

```json
{ "type": "registry/get", "key": "some-invalid-key-heeeeerrrreeee" }
{ "type": "error", "reason": "key not found" }

{ "type": "registry/get", "key": "workspace-right" }
{ "type": "error", "reason": "invalid key" }

{ "type": "registry/get", "key": "notify" }
{ "type": "error", "reason": "no write access"}
```

## registry.set
Set values from the registry.
```json
{ "type": "registry.set", "key": "foo", "value": "bar" }
```

### Reply
The reply will either be an empty `success` with the serialized value, or an access
error.
```json
{ "type": "registry.set", "key": "mouse.coords", "value": { "x": "12", "y": 22 } }
```

### Errors
Errors will be returned if the registry key cannot be accessed
(if it is not an object or if it has restricted flags).
```json
{ "type": "registry.set", "key": "private_key", "value": "secret" }
{ "type": "error", "reason": "invalid key" }
```

## registry.contains\_key
Checks if a registry key exists.
```json
{ "type": "registry.contains_key", "key": "some_key" }
```

### Reply
The reply will either be `"contains": "true", "type": <key-type>` or 
`"contains": "false"`. The `key_type` field is one of `object`, `property`, and `command`.

```json
{ "type": "registry.contains_key", "key": "some_key" }
{ "type": "success", "contains": "true", "key_type": "object"}

{ "type": "registry.contains_key", "key": "quit" }
{ "type": "success", "contains": "true", "key_type": "command" }

{ "type": "registry.contains_key", "key": "wm.pointer" }
{ "type": "success", "contains": "true", "key_type": "property" }

{ "type": "registry.contains_key", "key": "foobar" }
{ "type": "success", "contains": "false" }

```
### Errors
This command will not return errors.

## command.run
Run a command.
```json
{ "type": "command.run", "key": "workspace_left" }
```

### Reply
The reply will either be `success` or an error.
```lua
{ "type": "command.run", "key": "workspace_right" }
{ "type": "success" }
```

### Errors
An error will be returned if the command does not exist.
```lua
{ "type": "command.run", "key": "bogus_command" }
{ "type": "error", "reason": "key not found" }
```
