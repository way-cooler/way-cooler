#!/usr/bin/env python2

import sys
import os
import re

from semver import semver

"""way-cooler CI integration.

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
from docopt import docopt

VERSION_REGEX = '\\d+\\.\\d+\\.\\d+'
BRANCH_REGEX = 'release-(' + VERSION_REGEX + ')'
# If we grab the first 'version=' line in the Cargo files we'll be fine
CARGO_VERSION_LINE = '$version = "' + VERSION_REGEX + '"^'
README_CRATES_TAG = "crates\\.io/-v" + VERSION_REGEX + '-orange\\.svg'


def check_release_branch(branch_version):
    if verbose:
        print("Verifying requirements for release version " + str(version))
        print("Verifying Cargo.toml release...")



if __name__ == "__main__":
    args = docopt(__doc__, version="ci.py v1.0")
    verbose = args.v

    if args.travis_check:
        travis_pr_branch = os.environ["TRAVIS_PR_BRANCH"]
        if travis_pr_branch == "":
            print("Not running in a PR.")
            sys.exit(0)
        version_match = re.match(travis_pr_branch)
        if version_match == None:
            print("Not in a release branch PR.")
            sys.exit(0)
        try:
            version = semver.parse(args.version)
            check_release_branch(version, verbose)
        except ValueError as e:
            sys.stderr.write("Error parsing version in branch " + travis_pr_branch)
            sys.exit(1)

    elif args.bump:
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
