# Copyright 2012 Martin Pool
# Licensed under the Apache License, Version 2.0 (the "License").

"""Command-line interface"""

import argparse
import logging


from duralib.archive import Archive


_log = logging.getLogger('dura')


def _make_parser():
    """Make an ArgumentParser"""
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(
        title='commands')

    cp = subparsers.add_parser(
        'create-archive',
        help='Make a new archive to hold backups')
    cp.set_defaults(cmd_func=cmd_create_archive)
    cp.add_argument(
        'archive_directory',
        help='Local path to directory to be created')

    cp = subparsers.add_parser(
        'describe-archive',
        help='Show summary information about an archive')
    cp.set_defaults(cmd_func=cmd_describe_archive)
    cp.add_argument(
        'archive_directory',
        help='Local path to directory to be created')

    return parser


def cmd_create_archive(args):
    new_archive = Archive.create(args.archive_directory)
    _log.info("Created %s", new_archive)


def cmd_describe_archive(args):
    archive = Archive.open(args.archive_directory)
    _log.info("Opened archive %r", archive)


# Lazily initialized ArgumentParser.
_parser = None

def parse_command(argv):
    """Parse a command line; return parsed args.

    The returned args contain a cmd_func that can be called, passing the
    args, to actually run the command.
    """
    global _parser
    if _parser is None:
        _parser = _make_parser()
    args = _parser.parse_args(argv)
    _log.debug("cli args: %r", args)
    return args


def run_command(argv):
    args = parse_command(argv)
    args.cmd_func(args)
    return 0