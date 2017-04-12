# Screen
An interface that allows clients to query and manipulate the screens/outputs.

### List output UUIDs
`List() -> Array[String]`

Returns the arary of UUIDs that are associated with the outputs on the screen, ordered by when the output was made active.

### Resolution size of a screen
`Resolution(uuid: String) -> Struct(width: u32, height: u32)`

Gives the resolution of the output associated with the UUID. If the UUID is not associated with an output, then an error is returned.

The first element returned in the struct is the width, followed by the height. Both are represented as unsigned 32-bit numbers.

### Scrape the pixels on the screen
`Scrape() -> Array[u8]`

Scrapes the pixels on the active output, returning the raw byte buffers from the rendered surface.

Each unsigned 8-bit number is a byte, with collections of four being a single RGBA pixel. The length of the buffer will **always** be divisible by 4.

The order of the bytes within the pixel is a little backwards, compared to how some programs expect it. For example, in order for [way-cooler-grab](https://github.com/way-cooler/way-cooler-grab) to output the bytes into a png file it has to shift the bytes in the pixel over by one. See [here](https://github.com/way-cooler/way-cooler-grab/blob/master/src/main.rs#L116) for more information.
