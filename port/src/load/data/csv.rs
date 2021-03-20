use std::collections::BTreeMap;
use csv::{ReaderBuilder, Reader};
use common::error::Error;
use common::err;

pub fn load<R: std::io::Read>(reader: &mut Reader<R>, size_limit: usize) -> Result<Vec<BTreeMap<String, String>>, Error> {
    let mut hashmap_vec = Vec::new();
    let mut curr_size = 0;
    for result in reader.deserialize() {
        if curr_size == size_limit{
            break;
        }
        let result = match result  {
            Err(e)  => return err!("csv", format!("{:?}", e).as_str()),
            Ok(r) => r
        };

        let record: BTreeMap<String, String> = result;

        hashmap_vec.push(record);
        curr_size += 1;
    }
    Ok(hashmap_vec)
}

pub fn mk_reader<R: std::io::Read>(reader: R) -> Reader<R>{
    ReaderBuilder::new().from_reader(reader)
}