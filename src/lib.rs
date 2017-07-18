extern crate byteorder;
extern crate fs2;
#[macro_use] extern crate log;
extern crate memmap;
extern crate rand;

mod offsets;
mod search;
mod types;

pub use offsets::*;
pub use search::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use byteorder::*;
    use std::fs::File;
    use std::io::Write;
    use rand::random;
    use std::time::*;

    // beamlog: 25737 gens, 76464 bytes/gen
    #[test]
    fn beamlog() {
        let mut f = File::create("test_data/beamlogish.log").unwrap();
        for i in 0..25737 {
            let jitter = random::<u64>() % 20_000;
            let len = 66_464 + jitter;
            let now = get_time();
            // let len = i % 32;
            f.write_u64::<BigEndian>(24 + len).unwrap();
            f.write_u64::<BigEndian>(i*3).unwrap();
            f.write_u64::<BigEndian>(now).unwrap();
            f.write(&vec![0xff;len as usize]).unwrap();
        }
    }

    fn get_time() -> u64 {
        let foo = UNIX_EPOCH.elapsed().unwrap();
        foo.as_secs() * 1_000_000 + foo.subsec_nanos() as u64/1_000
    }
}
