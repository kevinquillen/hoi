use std::io::{BufReader, Read};
use serde::{Deserialize, Serialize};
use serde_yaml;
use serde_yaml::Mapping;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Hoi {
    version: i32,
    description: String,
    commands: Mapping
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Command {
    cmd: Vec<String>,
    usage: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // recursive look up to user home dir for .hoi.yml
    // parse file
    // if no commands: section, panic
    // if no arg passed, display all commands
    // if arg
    //   if cmd in commands
    //     OS exec cmd
    //   else
    //     panic, cmd not found

    // example routine
    let file = std::fs::File::open(".hoi.yml")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let hoi: Hoi = serde_yaml::from_str(&contents)?;
    println!("Current version in YAML: {:?}", hoi.version);
    println!("Current description in YAML: {:?}", hoi.description);
    println!("Current commands in YAML: {:?}", hoi.commands);
    Ok(())
}
