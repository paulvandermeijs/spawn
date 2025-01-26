use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub(crate) fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
