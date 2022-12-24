#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::BTreeMap;
use std::io;
use std::io::{BufReader, Read, Write};
use std::env;
use serde::{Deserialize};
use serde_yaml;
use tabled::{Tabled, Table, Style, Alignment, Modify, Full, Header, Footer};
use std::process::{Command, Stdio};

#[derive(Default, Deserialize)]
struct Hoi {
    #[serde(default = "default_version")]
    version: String,
    entrypoint: Vec<String>,
    commands: BTreeMap<String, UserCommand>
}

#[derive(Debug, Deserialize, Tabled)]
struct UserCommand {
    #[header(hidden = true)]
    cmd: String,
    #[header("Usage")]
    usage: String,
}

fn default_version() -> String {
    "1".to_string()
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

    let args: Vec<String> = env::args().skip(1).collect();
    let hoi: Hoi = serde_yaml::from_str(&contents)?;

    if args.is_empty() {
        let table = Table::new(hoi.commands)
            .with(Header("Hoi Commands"))
            .with(Modify::new(Full)
                .with(Alignment::left()))
            .with(Style::NO_BORDER)
            .to_string();
        println!("{}", table);
    } else {
        let selected = &hoi.commands[args.get(0).unwrap()];
        println!("Running command {}...", args.get(0).unwrap());
        println!("{:?}", args.get(1).unwrap());
        // let output = Command::new("echo")
        //     .args(["I see you", "aaaaa"])
        //     .stdout(Stdio::piped())
        //     .output()
        //     .expect("Failure message here");

        let output = Command::new("./test.sh")
            .args(["echo", args.get(1).unwrap()])
            .spawn()?
            .wait_with_output();

        //io::stdout().write_all(&output.stdout).unwrap();
        //println!("{:?}", String::from_utf8_lossy(&output.stdout).trim());
    }

    Ok(())
}
