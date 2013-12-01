// Conserve - robust backup system
// Copyright 2012-2013 Martin Pool
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

package main

import (
    "github.com/docopt/docopt.go"
    "github.com/sourcefrog/conserve"
)

const usage = `
conserve - a robust backup program

Copyright 2012-2013 Martin Pool
Licenced under the GNU General Public Licence, version 2 or later.
Conserve comes with ABSOLUTELY NO WARRANTY of any kind.

Usage:
  conserve backup <source>... <archive>
  conserve [-v] init <dir>
  conserve printproto <file>
  conserve restore <archive> <destdir>
  conserve validate <archive>

Options:
  --help        Show help.
  --version     Show version.
  -v            Be more verbose.
`

func main() {
    args, _ := docopt.Parse(usage, nil, true,
        conserve.ConserveVersion, false)

    if args["init"].(bool) {
        conserve.InitArchive(args["<dir>"].(string))
    }
}
