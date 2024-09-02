use std::io::{BufRead, BufReader, stdin};
use std::collections::HashMap;
use log::{info, warn, error};

struct PrimaryKeyMap {
    map: HashMap<String, u16>,
    reverse_map: Vec<String>,
}

impl PrimaryKeyMap {
    fn new() -> PrimaryKeyMap {
        PrimaryKeyMap {
            map: HashMap::new(),
            reverse_map: Vec::new(),
        }
    }

    fn key_for_value(&mut self, key: &str) -> u16 {
        match self.map.get(key) {
            Some(value) => *value,
            None => {
                let new_value = self.reverse_map.len() as u16;
                self.map.insert(key.to_string(), new_value);
                self.reverse_map.push(key.to_string());
                new_value
            }
        }
    }

    fn value_for_key(&self, key: u16) -> Option<&String> {
        self.reverse_map.get(key as usize)
    }
}

fn name_to_readname_key(name: &str, primary_key_map: &mut PrimaryKeyMap) -> [u16; 4] {
    let name_parts = name.split(':').collect::<Vec<&str>>();
    let mut key = [0 as u16; 4];
    key[0] = primary_key_map.key_for_value(&name_parts[0..4].join(":"));
    for i in 0..3 {
        key[i + 1] = primary_key_map.key_for_value(name_parts[4+i]);
    }
    return key;
}

fn main() {
    env_logger::init();
    let mut readname_part_pkm = PrimaryKeyMap::new();
    let mut comment_pkm = PrimaryKeyMap::new();
    let mut readnames_to_comments: HashMap<[u16; 4], u16> = HashMap::new();
    let mut args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("Usage: samtools view -h input.sam | {} <barcodefile> [barcodefile...] > output.sam\n", args[0]);
        eprintln!("This program reads a SAM file from stdin and adds the barcode information from the barcodefile(s) to the comments of the reads.\n");
        eprintln!("The barcodefile(s) should contain one line per read, with the read name followed by a space and the barcode sequence.");
        eprintln!("A good way to get that is to redirect a fastq file through awk to print just the header lines\n");
        eprintln!("samtools view -h input.sam | {} <(xzcat file.fastq.xz | awk 'NR % 4 == 1') > samtools view -Ocram -o output.cram", args[0]);
        std::process::exit(1);
    }

    for barcodefile in args.drain(1..) {
        let file = std::fs::File::open(barcodefile.clone()).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let read_name = parts[0];
            let readname_key = name_to_readname_key(read_name, &mut readname_part_pkm);
            let fastq_comment = parts[1];
            let fastq_comment = if fastq_comment.starts_with("1:N:0:") || fastq_comment.starts_with("2:N:0:") {
                format!("BC:Z:{}", &fastq_comment[6..])
            } else {
                warn!("Comment {} does not start with 1:N:0: or 2:N:0:, will use XC tag", fastq_comment);
                format!("XC:Z:{}", fastq_comment)
            };
            let comment_id = comment_pkm.key_for_value(&fastq_comment);
            let old_value = readnames_to_comments.insert(readname_key, comment_id);
            match old_value {
                Some(old_value) => {
                    if old_value != comment_id {
                        panic!("Identical readname key '{}' gave two different values: '{}' and '{}'",
                            read_name, comment_pkm.value_for_key(old_value).unwrap(), fastq_comment);
                    }
                },
                None => {}
            }
        }
        info!("Read comments for {} readnames after reading from {}", readnames_to_comments.len(), barcodefile);
        info!("Number of distinct readname parts 'interned': {}", readname_part_pkm.reverse_map.len());
        info!("Number of distinct comments 'interned': {}", comment_pkm.reverse_map.len());
    }

    for line in stdin().lock().lines() {
        let line = line.unwrap();
        if line.starts_with("@") {
            println!("{}", line)
        } else {
            let row = line.split('\t').collect::<Vec<&str>>();
            let name_key = name_to_readname_key(&format!("@{}", row[0]), &mut readname_part_pkm);
            match readnames_to_comments.get(&name_key) {
                Some(comment_key) => {
                    let comment = comment_pkm.value_for_key(*comment_key).unwrap();
                    if !comment.starts_with("BC:Z:") {
                        warn!("Comment looks wrong: {} for {}", comment, row[0]);
                    }
                    println!("{}\t{}", line, comment);
                },
                None => {
                    error!("No comment found for read name {}", row[0]);
                    // println!("{}\t{}", line, "XC:Z:UNKNOWN");
                }
            }
        }
    }
    
}
