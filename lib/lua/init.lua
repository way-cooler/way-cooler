-- Defines the global tables for Awesome compatibility

keygrabber = {
  -- Secret callback function that is called for every keyboard press.
  __callback = nil
}

mousegrabber = {
  -- Secret callback function that is called for every mouse event
  __callback = nil,
  -- Secret cursor icon that we don't really apply because this is Wayland
  __cursor = nil
}
