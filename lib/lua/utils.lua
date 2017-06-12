--------------------
--- Module with utility functions
--
--- @module utils
--
--- Submodules:
--
--- * `file`: Operations on files
--- * `math`: Contains a few math functions
--- * `string`: Operations on strings
--- * `table`: Operations on tables

util = {}

--- List of programs that should be spawned each start/restart.
util.program = {}
util.program.programs = {}

--- Spawns a program @param bin with the provided @param args.
--- Does not update the global program spawn list.
--- @param bin The program to run. Can be an absolute path or a command to run.
--- @param args The arguments (as a string) to pass to the program.
function util.program.spawn(bin, args)
  assert(type(bin) == 'string', 'Non string given for program')
  if type(args) ~= 'string' then
    args = ""
  end
  os.execute(bin .. " " .. args .. " &")
end

--- Returns a function that spawns a program once.
--- Does not update the global program spawn list.
--- Used primarily for key mapping.
--- @param bin The program to run. Can be an absolute path or a command to run.
--- @param args The arguments (as a string) to pass to the program.
--- @return Function that calls @param bin with @param args.
function util.program.spawn_once(bin, args)
  return function() util.program.spawn(bin, args) end
end

--- Registers the program to spawn at startup and every time it restarts
--- @param bin The program to run. Can be an absolute path or a command to run.
--- @param args The arguments (as a string) to pass to the program.
function util.program.spawn_at_startup(bin, args)
  assert(type(bin) == 'string', 'Non string given for program')
  table.insert(util.program.programs, {
                 bin = bin,
                 args = args
  })
end

--- Spawns the startup programs
function util.program.spawn_startup_programs()
  for index, program in ipairs(util.program.programs) do
    util.program.spawn(program.bin, program.args)
  end
end

--- Stops the startup programs. Does not remove them from the global list.
function util.program.terminate_startup_programs()
  for index, program in ipairs(util.program.programs) do
    -- TODO Kill in a more fine-grained matter...
    -- parent joining on child process? PIDs won't work, they can be reclaimed.
    os.execute("pkill " .. program.bin)
  end
end

--- Stops the startup programs and then immediately starts them again.
--- Useful for the "restart" command
function util.program.restart_startup_programs()
  util.program.terminate_startup_programs()
  util.program.spawn_startup_programs()
end
