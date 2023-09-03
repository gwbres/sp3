//! Buffered writer wrapper, for efficient data generation
use std::fs::File;
use std::io::{BufWriter, IoSlice, Write};

pub(crate) struct BufferedWriter {
    buf: BufWriter<File>,
}

impl BufferedWriter {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let fd = File::create(path)?;
        Ok(Self {
            buf: BufWriter::new(fd),
        })
    }
}

impl std::io::Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.buf.write_vectored(bufs)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
