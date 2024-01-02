// The MIT License (MIT)
//
// Copyright (c) 2023 Pedro Tacla Yamada
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::io::Write;

pub struct DefaultWriter {
    imp: Box<dyn std::io::Write>,
}

impl DefaultWriter {
    pub fn new(inplace: bool, input_file: &str) -> Self {
        DefaultWriter {
            imp: if inplace {
                Box::new(std::fs::File::create(input_file).expect("Failed to open file"))
            } else {
                Box::new(std::io::stdout())
            },
        }
    }
}

impl Write for DefaultWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf = String::from_utf8(buf.to_vec()).unwrap();
        let buf = buf.replace("\r", "");
        if buf.is_empty() {
            return Ok(0);
        }
        self.imp.write(buf.as_ref())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.imp.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let buf = String::from_utf8(buf.to_vec()).unwrap();
        let buf = buf.replace("\r", "");
        if !buf.is_empty() {
            let _ = self.imp.write(buf.as_ref());
        }
        return Ok(());
    }
}
