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

    fn get(&mut self, key: &str) -> u16 {
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

    fn get_key(&self, key: u16) -> Option<&String> {
        self.reverse_map.get(key as usize)
    }
}

fn name_to_key(name: &str, primary_key_map: &mut PrimaryKeyMap) -> [u16; 4] {
    let name_parts = name.split(':').collect::<Vec<&str>>();
    let mut key = [0 as u16; 4];
    key[0] = primary_key_map.get(&name_parts[0..4].join(":"));
    for i in 0..3 {
        key[i + 1] = primary_key_map.get(name_parts[4+i]);
    }
    return key;
}

fn main() {
    env_logger::init();
    let mut primary_key_map = PrimaryKeyMap::new();
    let mut readnames_to_comments: HashMap<[u16; 4], u16> = HashMap::new();
    let mut args = std::env::args().collect::<Vec<String>>();

    for barcodefile in args.drain(1..) {
        let file = std::fs::File::open(barcodefile.clone()).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let read_name = parts[0];
            let key = name_to_key(read_name, &mut primary_key_map);
            let fastq_comment = parts[1];
            let fastq_comment = if fastq_comment.starts_with("1:N:0:") || fastq_comment.starts_with("2:N:0:") {
                format!("BC:Z:{}", &fastq_comment[6..])
            } else {
                warn!("Comment {} does not start with 1:N:0: or 2:N:0:, will use XC tag", fastq_comment);
                format!("XC:Z:{}", fastq_comment)
            };
            readnames_to_comments.insert(key, primary_key_map.get(&fastq_comment));
        }
        info!("Read {} comments after reading from {}", readnames_to_comments.len(), barcodefile);
        info!("Number of strings 'interned': {}", primary_key_map.reverse_map.len());
    }

    for line in stdin().lock().lines() {
        let line = line.unwrap();
        if line.starts_with("@") {
            println!("{}", line)
        } else {
            let row = line.split('\t').collect::<Vec<&str>>();
            let key = name_to_key(&format!("@{}", row[0]), &mut primary_key_map);
            match readnames_to_comments.get(&key) {
                Some(comment_key) => {
                    let comment = primary_key_map.get_key(*comment_key).unwrap();
                    if !comment.starts_with("BC:Z:") {
                        warn!("Comment looks wrong: {} for {}", comment, row[0]);
                    }
                    println!("{}\t{}", line, comment);
                },
                None => {
                    error!("No comment found for read name {}", row[0]);
                    println!("{}\t{}", line, "XC:Z:UNKNOWN");
                }
            }
        }
    }
    
}
