// Copyright 2017, 2019, 2020 Martin Pool.

// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! Snappy compression glue.

use snap::raw::{Decoder, Encoder};

use crate::Result;

/// Holds a reusable buffer for Snappy compression.
pub(crate) struct Compressor {
    out_buf: Vec<u8>,
    encoder: Encoder,
}

impl Compressor {
    pub fn new() -> Compressor {
        Compressor::default()
    }

    /// Compress bytes into unframed Snappy data.
    ///
    /// Returns a slice referencing a buffer in this object, valid only
    /// until the next call.
    pub fn compress(&mut self, input: &[u8]) -> Result<&[u8]> {
        let max_len = snap::raw::max_compress_len(input.len());
        if self.out_buf.len() < max_len {
            self.out_buf.resize(max_len, 0u8);
        }
        let actual_len = self.encoder.compress(input, &mut self.out_buf)?;
        Ok(&self.out_buf[0..actual_len])
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Compressor {
            out_buf: Vec::new(),
            encoder: Encoder::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Decompressor {
    out_buf: Vec<u8>,
    decoder: Decoder,
    last_len: usize,
}

impl Decompressor {
    pub fn new() -> Decompressor {
        Decompressor::default()
    }

    /// Decompressed unframed Snappy data.
    ///
    /// Returns a slice pointing into a reusable object inside the Decompressor.
    pub fn decompress(&mut self, input: &[u8]) -> Result<&[u8]> {
        let max_len = snap::raw::decompress_len(input)?;
        if self.out_buf.len() < max_len {
            self.out_buf.resize(max_len, 0u8);
        }
        let actual_len = self.decoder.decompress(input, &mut self.out_buf)?;
        self.last_len = actual_len;
        Ok(&self.out_buf[..actual_len])
    }

    /// Deconstruct this Decompressor and return its buffer with the latest contents.
    pub fn take_buffer(self) -> Vec<u8> {
        let Decompressor { mut out_buf, .. } = self;
        out_buf.truncate(self.last_len);
        out_buf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compressor_decompressor() {
        let mut compressor = Compressor::new();
        let mut decompressor = Decompressor::new();

        let comp = compressor.compress(b"hello world").unwrap();
        assert_eq!(comp, b"\x0b(hello world");
        assert_eq!(decompressor.decompress(comp).unwrap(), b"hello world");

        let long_input = b"hello world, hello world, hello world, hello world";
        let comp = compressor.compress(long_input).unwrap();
        assert_eq!(comp, b"\x32\x30hello world, \x92\x0d\0");
        assert_eq!(decompressor.decompress(comp).unwrap(), &long_input[..]);
    }
}
