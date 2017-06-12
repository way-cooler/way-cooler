#!/usr/bin/env python3

DEBUG_COLOR = "[44m"
DEBUG_START = "[44m DEBUG"
DEBUG_END   = "[0m"

import sys
from time import sleep

def read_commands(f):
    """Reads the commands from the provided open file handler.
    Commands are denonted by the bytes "\x1B[44m" and the word "DEBUG"
    as the first thing on the line. Eg:

    [44m DEBUG [layout::commands] [37msrc/layout/commands.rs:367\
    [0m[44m Layout.SwitchWorkspace(1) [0m
    """
    commands = []
    for line in f:
        if line.startswith(DEBUG_START):
            start = line.find(DEBUG_COLOR, len(DEBUG_START)) + len(DEBUG_COLOR)
            end = line.find(DEBUG_END, start)
            commands.append(line[start: end].strip())
    return commands


def execute_commands(command_list):
    """Executes the commands in the command list.

    **DANGER**

    This is VERY unsafe, as it passing the strings directly to `exec`"""
    from pydbus import SessionBus
    bus = SessionBus()
    Layout = bus.get(bus_name="org.way-cooler",
                     object_path="/org/way_cooler/Layout")
    for command in command_list:
        print("Executing command: \"{}\"".format(command))
        sleep(1)
        exec(command)


if __name__ == "__main__":
    from docopt import docopt
    DEBUG_REPRODUCE_USAGE = """debug reproduce system for Way Cooler.

    Usage:
      debug_reproduce.py <file> [-p]

    Options:
      -p            print the commands, without executing them
      -h --help     show this menu
    """
    args = docopt(DEBUG_REPRODUCE_USAGE)
    log_path = sys.argv[1]
    commands = None
    with open(log_path, "r") as log_file:
        commands = read_commands(log_file);
    for command in commands:
        print(command)
    if not args["-p"]:
        execute_commands(commands)
