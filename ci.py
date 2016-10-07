#!/usr/bin/env python2

import sys

from semver import semver

"""way-cooler CI integration.

Usage:
  ci.py check <version> [-v]
  ci.py bump <old version> <new version> [-v]
  ci.py (-h | --help)
  ci.py --version

Options:
  -h --help    Show this menu
  -v           Be verbose, print actions taken
  --version    Show version information
"""
from docopt import docopt

VERSION_REGEX = '\d+\.\d+\.\d+'
"""If we grab the first 'version=' line in the Cargo files we'll be fine."""
CARGO_VERSION_LINE = '$version = "' + VERSION_REGEX + '"^'
README_CRATES_TAG = ""

def check_version(semver, verbose):
    if verbose:
        print("Verifying requirements for release version " + str(version))
        print("Verifying Cargo.toml release...")



if __name__ == "__main__":
    args = docopt(__doc__, version="ci.py v1.0")
    verbose = args.v
    if verbose:
        print(args)

    if args.check:
        try:
            version = semver.parse(args.version)
            check_version(version, verbose)
        except ValueError as e:
            sys.stderr.write("Invalid version %s.\n" % args.version)
            exit(1)

    else if args.bump:
        try:
            old_version = semver.parse(args["old version"])
            new_version = semver.parse(args["new version"])
            bump_version(old_version, new_version, verbose)
        except ValueError as e:
            sys.stderr.write(
                "Invalid version %s or %s.\n" % args["old_version"], args["new_version"])
            exit(1)

    else:
        sys.stderr.write("Invalid arguments!\n")
        exit(1)
