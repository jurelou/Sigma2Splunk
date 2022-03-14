use clap::{Command, AppSettings, Arg, ArgMatches};
use std::path::{Path, PathBuf};
use anyhow::{Result};


struct Sigma2Splunk {
    threads: usize,
    username: String,
    password: String,
    index: String,
    rules: PathBuf
}

impl Sigma2Splunk {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self> {

        let rules = PathBuf::from(
            matches
                .value_of("RULES")
                .expect(""),
        );
        let username = matches.value_of("username").unwrap().to_string();
        let password = matches.value_of("password").unwrap().to_string();
        let index = matches.value_of("index").unwrap().to_string();

        let threads = matches
            .value_of("threads")
            .map(|value| value.parse::<usize>().expect(""))
            .unwrap();
        println!("Using {:?} threads", threads);

        Ok(Sigma2Splunk {
            threads,
            username,
            password,
            index,
            rules
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("aaa{:?}", self.username);
        Ok(())
    }
}

fn is_uint(value: &str) -> Result<(), String> {
    match value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Expected value to be a positive number.".to_owned()),
    }
}

fn main() -> Result<()> {
    let matches = Command::new("Sigma 2 Splunk")
        .setting(AppSettings::DeriveDisplayOrder)
        .author("ljk")
        .about("Run sigma queries against a splunk instance.")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::new("RULES").required(true))
        .arg(
            Arg::new("username")
                .long("username")
                .short('u')
                .takes_value(true)
                .help("Splunk username")
                .required(true),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .short('p')
                .takes_value(true)
                .help("Splunk password")
                .required(true),
        )
        .arg(
            Arg::new("index")
                .long("index")
                .short('i')
                .takes_value(true)
                .default_value("main")
                .help("Splunk index to use for queries"),
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .short('t')
                .default_value("4")
                .help("Number of parallel requests")
                .validator(is_uint),
        )
        .get_matches();

        Sigma2Splunk::from_matches(&matches)?.run()?;
        Ok(())
}