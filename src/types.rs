/// The sequence number of a message. The first message in the file has `SeqNum(0)`.
#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub struct SeqNum(pub u64);

/// A byte offset into the file.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ByteOffset(pub u64);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Target { pub field: u64, pub value: u64 }
