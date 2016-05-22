# Sockets
IPC with way-cooler taks place over two Unix sockets:

`/tmp/way-cooler/server` and `/tmp/way-cooler/events`.

Requests in the `server` socket allow clients to send commands to the server and fetch specific data.

The `events` socket allows clients to subscribe to events (key presses, workspace switches, etc.) from way-cooler. 

Communication in both pipes done by exchanging a series of "packets" back and forth
between the client and server.

# Packets
A packet will consist of a length followed by JSON payload.
The length shall be an unsigned 32-bit number (`uint32`) encoded in big-endian.
Following this will be a UTF-8 encoded string representing a valid JSON object.

Replies from the server will follow the same protocol.

# Command Channel Packets
These are the packets used in `/tmp/way-cooler/server`. 
Each is a JSON table, with a `type` field specifying what kind of packet it is.

## Errors
In the event of an error - a malformed packet or an invalid user action, an error response will be sent.
This has the key `type` set to `error` and a short description in `reason`.

```json
SEND { "type": "some-invalid-type" }
RECV { "type": "error", "reason": "invalid message type" }
```

# Communication
The client starts communication by sending a packet, and the server will send responses.
Replies either have `"type": "success"` or `"type": "error"`.

## get
Gets data from way-cooler. See the <registry docs> for a list of keys.
```json
{ "type": "get", "key": "views.current" }
{ "type": "get", "key": "mouse.coords" }
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
SEND { "type": "get", "key": "some-invalid-key-heeeeerrrreeee" }
RECV { "type": "error", "reason": "key not found" }

SEND { "type": "get", "key": "workspace-right" }
RECV { "type": "error", "reason": "invalid key" }

SEND { "type": "get", "key": "notify" }
RECV { "type": "error", "reason": "no write access"}
```

## set
Sets data in way-cooler. See <the registry docs> for a list of keys.
```json
{ "type": "set", "key": "mouse.coords", "value": { "x": 12, "y": 22 } }
```

### Reply
The reply will either be an empty `success` with the serialized value, or an access
error.
```json
SEND { "type": "set", "key": "mouse.coords", "value": { "x": 12, "y": 22 } }
RECV { "type": "success" }
```

### Errors
Errors will be returned if the registry key cannot be accessed
(if it is not an object or if it has restricted flags).
```json
SEND { "type": "set", "key": "private_key", "value": "secret" }
RECV { "type": "error", "reason": "invalid key" }
```

## exists
Checks if a key with that name exists
```json
{ "type": "exists", "key": "some_key" }
```

### Reply
The reply will either be `"contains": "true"` with fields `key_type` and `flags` or 
`"contains": "false"`. 
The `flags` field is a list possibly containing `"read"` and/or `"write"`, if `key_type` is not `command`.
The `key_type` field is one of `object`, `property`, and `command`.

```json
SEND { "type": "exists", "key": "some_key" }
RECV { "type": "success", "contains": "true", "key_type": "object", "flags": [ "read", "write" ] }

SEND { "type": "exists", "key": "quit" }
RECV { "type": "success", "contains": "true", "key_type": "command" }

SEND { "type": "exists", "key": "pointer.coords" }
RECV { "type": "success", "contains": "true", "key_type": "property", "flags": [ "read", "write" ] }

SEND { "type": "exists", "key": "foobar" }
RECV { "type": "success", "contains": "false" }
```
### Errors
This command will not return errors.

## run
Run a command. Comamdns are considered different than data: they are actions which take and return no data.
```json
{ "type": "run", "key": "workspace_move_left" }
```

### Reply
The reply will either be `success` or an error.
```json
SEND { "type": "run", "key": "workspace_send_right" }
RECV { "type": "success" }
```

### Errors
An error will be returned if the command does not exist.
```json
SEND { "type": "run", "key": "bogus_command" }
RECV { "type": "error", "reason": "key not found" }
```

### version
Gets the version of the API. Currently version 0, will hit version 1 on release,
and any changes in the API result in a version increment. Consult these docs to see
what changes between versions.

```json
SEND { "type": "version" }
RECV { "type": "success", "value": 1 }
```

# Events
By connecting to an event channel, clients can recieve envents from way-cooler. 

## Requesting
The first pakcet set across the event pipe should be a list of events requested.
```json
{ "type": "request", "events": [  { "type": "key.pressed", "keys": [ "ctrl", "alt", "delete" ] }, "workspace.switched" ] }
```
If the request packet is formatted properly, the server wil send a `{ "type": "success" }` packet.

### Errors
The reply will be an error if the JSON was invalid or an event was requested which does not exist.
The client may re-attempt to register events after receiving an error.

## Event loop
After the server sends the success packet, it will send events. The server will not listen for inputs and only send more packets.

### Event packets
Event packets are described in the <event docs>. The general format:
```json
{ "type": "event", "name": "event_name", "field1": "value",  }
```
