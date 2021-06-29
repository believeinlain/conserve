// Conserve backup system.
// Copyright 2021 Martin Pool.

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

use std::path::PathBuf;

use conserve::test_fixtures::testdata_archives;

#[test]
#[should_panic(expected = "read testdata dir failed")]
fn testdata_archives_nonexistent_panics() {
    testdata_archives("nonexistent");
}

#[test]

fn testdata_archives_simple() {
    assert_eq!(
        testdata_archives("simple"),
        vec![PathBuf::from("testdata/archive/simple/v0.6.10")]
    );
}
