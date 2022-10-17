// Copyright 2022 Stephanie Aelmore.
// Copyright 2017 Julian Raufelder.
// Copyright 2020, 2021, 2022 Martin Pool.

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! Specify which files to include in the archive, rather than including all
//! by default or relying on specified excludes.
//!
//! Globs match against Apaths.
//!
//! Patterns that start with a slash match only against full paths from the top
//! of the tree. Patterns that do not start with a slash match the suffix of the
//! path.

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use super::*;

/// Describes which files to include in a backup
#[derive(Clone, Debug)]
pub struct Include {
    globset: GlobSet,
    // TODO: Control of matching cachedir.
}

impl Include {
    /// Create an [Include] from a list of glob strings.
    ///
    /// The globs match against the apath, which will
    /// always start with a `/`.
    pub fn from_strings<I: IntoIterator<Item = S>, S: AsRef<str>>(includes: I) -> Result<Include> {
        let mut builder = IncludeBuilder::new();
        for s in includes {
            builder.add(s.as_ref())?;
        }
        builder.build()
    }

    /// Include all items not specifically excluded
    pub fn all() -> Include {
        IncludeBuilder::new()
            .add("/**/*")
            .unwrap()
            .build()
            .expect("Unable to build default 'include all' glob.")
    }

    /// True if this apath should be excluded.
    pub fn matches<'a, A>(&self, apath: &'a A) -> bool
    where
        &'a A: Into<Apath> + 'a,
        A: ?Sized,
    {
        let apath: Apath = apath.into();
        self.globset.is_match(apath)
    }
}

/// Construct Include object.
pub struct IncludeBuilder {
    gsb: GlobSetBuilder,
}

impl IncludeBuilder {
    pub fn new() -> IncludeBuilder {
        IncludeBuilder {
            gsb: GlobSetBuilder::new(),
        }
    }

    pub fn build(&self) -> Result<Include> {
        Ok(Include {
            globset: self.gsb.build()?,
        })
    }

    pub fn add(&mut self, pat: &str) -> Result<&mut IncludeBuilder> {
        let pat: Cow<str> = if pat.starts_with('/') {
            Cow::Borrowed(pat)
        } else {
            Cow::Owned(format!("**/{}", pat))
        };
        let glob = GlobBuilder::new(&pat)
            .literal_separator(true)
            .build()
            .map_err(|source| Error::ParseGlob { source })?;
        self.gsb.add(glob);
        Ok(self)
    }

    pub fn add_file(&mut self, path: &Path) -> Result<&mut IncludeBuilder> {
        self.add_from_read(&mut File::open(path)?)
    }

    /// Create a [GlobSet] from lines in a file, with one pattern per line.
    ///
    /// Lines starting with `#` are comments, and leading and trailing whitespace is removed.
    pub fn add_from_read(&mut self, f: &mut dyn Read) -> Result<&mut IncludeBuilder> {
        let mut b = String::new();
        f.read_to_string(&mut b)?;
        for pat in b
            .lines()
            .map(str::trim)
            .filter(|s| !s.starts_with('#') && !s.is_empty())
        {
            self.add(pat)?;
        }
        Ok(self)
    }

    /// Build from command line arguments of patterns and filenames.
    pub fn from_args(include: &[String], include_from: &[String]) -> Result<IncludeBuilder> {
        let mut builder = IncludeBuilder::new();
        for pat in include {
            builder.add(pat)?;
        }
        for path in include_from {
            builder.add_file(Path::new(path))?;
        }
        Ok(builder)
    }
}

impl Default for IncludeBuilder {
    fn default() -> Self {
        IncludeBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn simple_globs() {
        let vec = vec!["fo*", "foo", "bar*"];
        let include = Include::from_strings(&vec).unwrap();

        // Matches in the root
        assert!(include.matches("/foo"));
        assert!(include.matches("/foobar"));
        assert!(include.matches("/barBaz"));
        assert!(!include.matches("/bazBar"));

        // Also matches in a subdir
        assert!(include.matches("/subdir/foo"));
        assert!(include.matches("/subdir/foobar"));
        assert!(include.matches("/subdir/barBaz"));
        assert!(!include.matches("/subdir/bazBar"));
    }

    #[test]
    fn rooted_pattern() {
        let include = Include::from_strings(&["/exc"]).unwrap();

        assert!(include.matches("/exc"));
        assert!(!include.matches("/excellent"));
        assert!(!include.matches("/sub/excellent"));
        assert!(!include.matches("/sub/exc"));
    }

    #[test]
    fn path_parse() {
        let include = Include::from_strings(&["fo*/bar/baz*"]).unwrap();
        assert!(include.matches("/foo/bar/baz.rs"))
    }

    #[test]
    fn extended_pattern_parse() {
        // Note that these are globs, not regexps, so "fo?" means "fo" followed by one character.
        let include = Include::from_strings(&["fo?", "ba[abc]", "[!a-z]"]).unwrap();
        assert!(include.matches("/foo"));
        assert!(!include.matches("/fo"));
        assert!(include.matches("/baa"));
        assert!(include.matches("/1"));
        assert!(!include.matches("/a"));
    }

    #[test]
    fn all_parse() {
        let include = Include::all();
        assert!(include.matches("/a"));
        assert!(include.matches("/.things"));
        assert!(include.matches("/stuff/and/things"));
        assert!(include.matches("/.stuff/and/things"));
        assert!(include.matches("/.stuff/and/things/.conf"));
    }
}
