// Conserve backup system.
// Copyright 2015, 2016 Martin Pool.

//! "Apaths" (for archive paths) are platform-independent relative file paths used inside archive
//! snapshots.
//!
//! The format and semantics of apaths are defined in ../doc/format.md.
//!
//! Apaths in memory are simply strings.

use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::fmt;
use std::fmt::{Display, Formatter};


#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Apath(String);


impl Apath {
    pub fn from_string(s: &str) -> Apath {
        assert!(valid(s));
        Apath(s.to_string())
    }

    pub fn to_string(&self) -> &String {
        &self.0
    }
}


impl Display for Apath {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.to_string())
    }
}


impl PartialEq<str> for Apath {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}


/// Compare two apaths.
impl Ord for Apath {
    fn cmp(&self, b: &Apath) -> Ordering {
        let &Apath(ref a) = self;
        let &Apath(ref b) = b;
        let mut ait = a.split('/');
        let mut bit = b.split('/');
        let mut oa = ait.next().expect("paths must not be empty");
        let mut ob = bit.next().expect("paths must not be empty");
        loop {
            return match (ait.next(), bit.next(), oa.cmp(ob)) {
                // Both paths end here: eg ".../aa" < ".../zz"
                (None, None, cmp) => cmp,

                // If one is a direct child and the other is in a subdirectory,
                // the direct child comes first.
                // eg ".../zz" < ".../aa/bb"
                (None, Some(_bc), _) => return Ordering::Less,
                (Some(_ac), None, _) => return Ordering::Greater,

                // If parents are the same and both have children keep looking.
                (Some(ac), Some(bc), Ordering::Equal) => {
                    oa = ac;
                    ob = bc;
                    continue;
                },

                // Both paths have children but they differ at this point.
                (Some(_), Some(_), cmp) => cmp,
            }
        }
    }
}

impl PartialOrd for Apath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


/// True if this apath is well-formed.
///
/// Rust strings are by contract always valid UTF-8, so to meet that requirement for apaths it's
/// enough to use a checked conversion from bytes or an `OSString`.
pub fn valid(a: &str) -> bool {
    if ! a.starts_with('/') {
        return false;
    } else if a.len() == 1 {
        return true;
    }
    for part in a[1..].split('/') {
        if part.is_empty()
            || part == "." || part == ".."
            || part.contains('\0') {
            return false;
        }
    }
    true
}


#[cfg(test)]
mod tests {
    use super::valid;
    use super::Apath;

    #[test]
    pub fn invalid() {
        let invalid_cases = [
            "",
            "//",
            "//a",
            "/a//b",
            "/a/",
            "/a//",
            "./a/b",
            "/./a/b",
            "/a/b/.",
            "/a/./b",
            "/a/b/../c",
            "../a",
            "/hello\0",
        ];
        for v in invalid_cases.into_iter() {
            if valid(v) {
                panic!("{:?} incorrectly marked valid", v);
            }
        }
    }

    #[test]
    pub fn valid_and_ordered() {
        let ordered = [
            "/",
            "/...a",
            "/.a",
            "/a",
            "/b",
            "/kleine Katze Fuß",
            "/~~",
            "/ñ",
            "/a/...",
            "/a/..obscure",
            "/a/.config",
            "/a/1",
            "/a/100",
            "/a/2",
            "/a/añejo",
            "/a/b/c",
            "/b/((",
            "/b/,",
            "/b/A",
            "/b/AAAA",
            "/b/a",
            "/b/b",
            "/b/c",
            "/b/a/c",
            "/b/b/c",
            "/b/b/b/z",
            "/b/b/b/{zz}",
        ];
        for (i, a) in ordered.iter().enumerate() {
            if !valid(a) {
                panic!("{:?} incorrectly marked invalid", a);
            }
            let ap = Apath::from_string(a);
            for (j, b) in ordered.iter().enumerate() {
                let expected_order = i.cmp(&j);
                let bp = Apath::from_string(b);
                let r = ap.cmp(&bp);
                if r != expected_order {
                    panic!("cmp({:?}, {:?}): returned {:?} expected {:?}",
                        ap, bp, r, expected_order);
                }
            }
        };
    }
}
