extern crate byteorder;
extern crate clap;
extern crate env_logger;
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

use env_logger::LogBuilder;
use log::LogLevelFilter;
use std::process::exit;
use std::path::{Path, PathBuf};

fn main() {
    // Parse the command-line arguments
    let args = clap::App::new("logsearch")
        .version("0.1")
        .about("Search a log file for a value")
        .arg(clap::Arg::from_usage("-f --field=[bytes] 'A byte-offset into messages, to identify a field'"))
        .arg(clap::Arg::from_usage("-t --target=[num] 'The target field value to search for'"))
        .arg(clap::Arg::from_usage("--index-file=[file] 'Where to cache the index'"))
        .arg(clap::Arg::from_usage("<file> 'The file to search'"))
        .arg(clap::Arg::from_usage("[verbosity]... -v 'Sets the level of verbosity'"))
        .after_help("logsearch  takes  a log-formatted file and searches for a particular message.
                     If the target message is found, the byte-offset of that message within the log
                     file is returned. See the man page for more information.")
        .get_matches();

    // Initialise the logger (prints errors to stderr)
    let log_level = log_level_from_int(args.occurrences_of("verbosity"));
    LogBuilder::new().filter(None, log_level).init().expect("Failed to start logger");
    debug!("Done parsing CLI and initialising logger");

    let log_path = args.value_of("file").map(Path::new).unwrap();
    let idx_path = args.value_of("index-file").map(PathBuf::from).unwrap_or(
        PathBuf::from((log_path.to_string_lossy() + ".idx").into_owned()));
    let idx = MsgOffsets::load(log_path, &idx_path);
    debug!("Done loading msg offsets");

    match (args.value_of("target"), args.value_of("field")) {
        (Some(value), Some(field)) => {
            let target = Target {
                field: field.parse().unwrap(),
                value: value.parse().expect("Parse target"),
            };
            info!("Searching for the message such that body[{}..{}] = {} in {:?}", target.field*8, target.field*8 + 8, target.value, log_path);
            let msg = binary_search(log_path, &idx, target);
            let offset = msg.and_then(|x| idx.lookup(x));
            info!("{:?}: {:?} => {:?} => {:?}", log_path, target, msg, offset);
            match offset { None => exit(1), Some(x) => { println!("{}", x.0); exit(0); } }
        }
        (Some(target), None) => {
            let target = SeqNum(target.parse().expect("Parse target"));
            info!("Searching for message no {:?} in {:?}", target, log_path);
            let offset = idx.lookup(target);
            info!("{:?}: {:?} => {:?}", log_path, target, offset);
            match offset { None => exit(1), Some(x) => { println!("{}", x.0); exit(0); } }
        }
        (None, _) => {
            info!("No target specified, skipping search. (Index will be updated though)");
        }
    }
}

fn log_level_from_int(n: u64) -> LogLevelFilter {
    match n {
        0 => LogLevelFilter::Off,
        1 => LogLevelFilter::Error,
        2 => LogLevelFilter::Warn,
        3 => LogLevelFilter::Info,
        4 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace,
    }
}

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
