use byteorder::*;
use fs2::FileExt;
use memmap::{Mmap, Protection};
use std::fs::{OpenOptions, File};
use std::io::{Seek, SeekFrom};
use std::path::Path;
use types::*;

/// A mapping from a message's sequence number to its byte offset in the log file.
///
/// A `MsgOffsets` index is always backed by a file. The file is a cache, and may be empty or
/// contain only a prefix of the messages. The file must be stored on a filesystem capable of
/// backing memory maps (ie. be careful with NFS).
//
// TODO: Make the index density configurable. Currently it's 1, but it could easily be 1/2, 1/3
// etc. at the expense of doing more reads/seeks.
#[derive(Debug)]
pub struct MsgOffsets(Mmap);

/// We assume that the given index file correctly maps sequence numbers to offsets into the given
/// log file, up to a certain message, but that the log may contain new data which was appended to
/// it since the index was last written. This function brings the index up-to-date by starting
/// where the index leaves off and, from there, jumping through the log file to find the offsets of
/// subsequent messages. These offsets are written back to the index file.
fn update_index(log_path: &Path, idx_file: &mut File) {
    let mut log_file = File::open(log_path).unwrap();
    let last_offset = match idx_file.seek(SeekFrom::End(-8)) {
        Ok(_) => idx_file.read_u64::<BigEndian>().expect("Read last entry in index file"),
        Err(_) => 0,
    };
    let new_data = log_file.metadata().expect("Query log file metadata").len() - last_offset;
    if new_data < 8 {
        info!("The index file is already up-to-date");
    } else {
        info!("The log file has grown by {} bytes since the index was last written. Updating...", new_data);
        log_file.seek(SeekFrom::Start(last_offset)).expect("Seek to last offset");
        loop {
            if let Ok(len) = log_file.read_u64::<BigEndian>() {
            if let Ok(offset) = log_file.seek(SeekFrom::Current(len as i64 - 8)) {
                idx_file.write_u64::<BigEndian>(offset).expect("Write entry to index file");
            } else {
                break;
            } } else { break; }
        }
    }
}

impl MsgOffsets {
    /// Load an index from a file, updating it if necessary. The file is created if it doesn't
    /// exist already.
    pub fn load(log_path: &Path, idx_path: &Path) -> MsgOffsets {
        let mut idx_file = OpenOptions::new()
            .read(true).append(true).create(true)
            .open(idx_path).expect("Open index file");
        idx_file.lock_exclusive().expect("Lock index file"); // Try to make mmaping safer
        update_index(log_path, &mut idx_file);
        let msg_offsets = MsgOffsets(Mmap::open(&idx_file, Protection::Read).unwrap());
        info!("Done loading msg offsets (last message: {:?})", msg_offsets.last());
        msg_offsets
    }

    pub fn lookup(&self, msg: SeqNum) -> Option<ByteOffset> {
        if msg > self.last() { None } else {
        // This is unsafe if the index file is modified concurrently. We make an effort to prevent
        // this by taking a flock. (See `load`).
        unsafe {
            let bs = self.0.as_slice();
            let i = msg.0 as usize * 8;
            let off = BigEndian::read_u64(&bs[i..i+8]);
            Some(ByteOffset(off))
        }
        }
    }

    pub fn last(&self) -> SeqNum {
        SeqNum(((self.0.len() / 8) - 1) as u64)
    }
}
