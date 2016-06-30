# Sockets
IPC with way-cooler taks place over two Unix sockets:

`/tmp/way-cooler/server` and `/tmp/way-cooler/events`.

Requests in the `server` socket allow clients to send commands to the server and fetch specific data.

The `events` socket allows clients to subscribe to events (key presses, workspace switches, etc.) from way-cooler.

Communication in both pipes done by exchanging a series of "packets" back and forth
between the client and server.

**The events socket is not yet implemented.**

# Packets
A packet will consist of a length followed by JSON payload.
The length shall be an unsigned 32-bit number (`uint32`) encoded in big-endian.
Following this will be a UTF-8 encoded string representing a valid JSON object.

Replies from the server will follow the same protocol.
