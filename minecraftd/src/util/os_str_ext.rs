use std::ffi::{OsStr, OsString};

pub trait OsStrExt {
    fn replace(&self, from: &OsStr, to: &OsStr) -> OsString;
}

impl OsStrExt for OsStr {
    fn replace(&self, from: &OsStr, to: &OsStr) -> OsString {
        let haystack = self.as_encoded_bytes();
        let needle = from.as_encoded_bytes();
        if needle.is_empty() {
            return self.to_os_string();
        }

        let mut out = Vec::with_capacity(haystack.len());
        let mut i = 0;

        while i + needle.len() <= haystack.len() {
            if &haystack[i..i + needle.len()] == needle {
                out.extend_from_slice(to.as_encoded_bytes());
                i += needle.len();
            } else {
                out.push(haystack[i]);
                i += 1;
            }
        }

        out.extend_from_slice(&haystack[i..]);

        unsafe { std::ffi::OsString::from_encoded_bytes_unchecked(out) }
    }
}
