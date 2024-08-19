use std::io::BufRead;
use std::collections::HashMap;

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
    let stdin = std::io::stdin();
    let mut primary_key_map = PrimaryKeyMap::new();
    let mut readnames_to_comments: HashMap<[u16; 4], u16> = HashMap::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let read_name = parts[0];
        let fastq_comment = parts[1];
        let key = name_to_key(read_name, &mut primary_key_map);
        readnames_to_comments.insert(key, primary_key_map.get(fastq_comment));
    }
    // just making sure get_key works
    println!("{}", primary_key_map.get_key(0).unwrap());
    println!("Number of strings 'interned': {}", primary_key_map.reverse_map.len());
    println!("Read names to comments size: {:?}", readnames_to_comments.len());
}
