#!/usr/bin/env python2

import sys
import os
import re

from docopt import docopt

VERSION_REGEX = '(\\d+\\.\\d+\\.\\d+)'
BRANCH_REGEX = '$release*' + VERSION_REGEX + '^'
# If we grab the first 'version=' line in the Cargo files we'll be fine
CARGO_VERSION_LINE = '$version = "' + VERSION_REGEX + '"^'
README_CRATES_TAG = "crates\\.io/-v" + VERSION_REGEX + '-orange\\.svg'

FILE_MAP = [
    ["Cargo.toml", CARGO_VERSION_LINE],
    ["Cargo.lock", CARGO_VERSION_LINE],
    ["README.md", README_CRATES_TAG]
]

DOCOPT_USAGE = """way-cooler CI integration.

Usage:
  ci.py travis-check
  ci.py bump <old version> <new version> [-v]
  ci.py (-h | --help)
  ci.py --version

Options:
  -h --help    Show this menu
  -v           Be verbose, print actions taken
  --version    Show version information
"""

failed = False

def check_file_version(file_name, regex, expected):
    reg = re.compile(regex)
    with open(file_name) as f:
        for line in f.readlines():
            match = reg.match(line)
            if match == None:
                continue
            elif match == expected:
                print('\t' + file_name + " updated.")
                return
            else:
                print('\t' + file_name + ": expected " + expected + ", got " + match)
                global failed
                failed = True

def check_release_branch(version):
    for mapping in FILE_MAP:
        print("Checking " + mapping[0])
        check_file_version(mapping[0], mapping[1], version)
    if failed:
        print("Some files not up to date!")
        sys.exit(2)

if __name__ == "__main__":
    args = docopt(DOCOPT_USAGE, version="ci.py v1.0")
    if args["travis-check"]:
        travis_pr_branch = os.environ["TRAVIS_PULL_REQUEST_BRANCH"]
        if travis_pr_branch == "":
            print("Not running in a PR.")
            sys.exit(0)
        version_match = re.match(travis_pr_branch)
        if version_match == None:
            print("Not in a release branch PR.")
            sys.exit(0)
        print("Checking versions in branch " + travis_pr_branch)
        check_release_branch(version_match)

    elif args["bump"]:
        sys.stderr.write("Not supported yet")
        sys.exit(1)

    else:
        sys.stderr.write("Invalid arguments!\n")
        exit(1)
