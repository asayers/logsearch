use byteorder::*;
use offsets::*;
use std::cmp::{Ord, Ordering};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;
use types::*;

/// Lookup the value for the given message and compare it to the target
fn check_msg(log_file: &mut File, idx: &MsgOffsets, msg: SeqNum, target: Target) -> Ordering {
    let offset = idx.lookup(msg).expect("Lookup message offset");
    log_file.seek(SeekFrom::Start(offset.0 + 8 + target.field)).expect("Seek to value of field");
    let x = log_file.read_u64::<BigEndian>().expect("Read value of field");
    debug!("binary search: {:?}[{}] => {}", msg, target.field, x);
    x.cmp(&target.value)
}

pub fn binary_search(log_path: &Path, idx: &MsgOffsets, target: Target) -> Option<SeqNum> {
    // let bounds = index.lookup(target);
    let mut log = File::open(log_path).unwrap();

    // First we check to see if target is in the file yet. If not, return None.
    let mut foo = idx.last();
    foo.0 -= 1;
    match check_msg(&mut log, idx, foo, target) {
        Ordering::Less  => return None,
        Ordering::Equal => return Some(idx.last()),
        Ordering::Greater => {},
    }

    // Ok, it's somewhere in the file. Let's binary search for it.
    let mut bounds = (SeqNum(0), idx.last());
    while (bounds.1).0 - (bounds.0).0 > 1 {
        let mid = SeqNum(((bounds.0).0 + (bounds.1).0) / 2);
        match check_msg(&mut log, idx, mid, target) {
            Ordering::Less  => bounds.0 = mid,
            Ordering::Equal => { bounds.0 = mid; bounds.1 = mid },
            Ordering::Greater => bounds.1 = mid,
        }
    }
    Some(bounds.1)
}
