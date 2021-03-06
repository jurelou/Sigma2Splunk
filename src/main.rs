use clap::{AppSettings, Arg, ArgMatches};
use std::path::PathBuf;
use anyhow::Result;
use std::fmt;
use std::error::Error;
use walkdir::WalkDir;
use std::process::Command;
use yaml_rust::YamlLoader;
use http_auth_basic::Credentials;
use std::{thread, time::Duration};


#[derive(Debug)]
struct InvalidFile {
    file: String
}

impl InvalidFile {
    fn new(file: &str) -> InvalidFile {
        InvalidFile{file: file.to_string()}
    }
}

impl fmt::Display for InvalidFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} does not exists !",self.file)
    }
}

impl Error for InvalidFile {
    fn description(&self) -> &str {
        &self.file
    }
}

struct Sigma2Splunk {
    threads: usize,
    username: String,
    password: String,
    index: String,
    splunk: String,
    earliest: String,
    rules: PathBuf
}

impl Sigma2Splunk {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, InvalidFile> {

        let rules = PathBuf::from(
            matches
                .value_of("RULES")
                .expect(""),
        );
        if !rules.exists() {
            let rules_str = rules.into_os_string().into_string().unwrap();
            return Err(InvalidFile::new(&rules_str));    
        }

        let username = matches.value_of("username").unwrap().to_string();
        let password = matches.value_of("password").unwrap().to_string();
        let index = matches.value_of("index").unwrap().to_string();
        let splunk = matches.value_of("splunk").unwrap().to_string();
        let earliest = matches.value_of("earliest").unwrap().to_string();

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
            splunk,
            earliest,
            rules
        })
    }

    fn run_query<P: Into<PathBuf>>(&self, rule: P) -> Result<()> {
        let file = rule.into();
        let output = Command::new("sigma/sigmac")
            .args(["-t", "splunk", "-c", "sigma/config.yml", &file.clone().into_os_string().into_string().unwrap()])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        if stdout.is_empty() {
            println!("Could not generate a rule from {:?}", file)
        } else {
            let rule_content = std::fs::read_to_string(&file).unwrap();
            let rule = &YamlLoader::load_from_str(&rule_content).unwrap()[0];

            let mut tags = Vec::new();
            
            for tag in rule["tags"].as_vec().unwrap() {
                tags.push(tag.as_str().unwrap())
            }
            println!("===");
            let query = format!("search index={} earliest=-{} {} | eval rule_name=\"{}\", tags=\"{}\" | collect index=alertes output_format=hec", 
                self.index,
                self.earliest,
                stdout.trim(),
                rule["title"].as_str().unwrap(),
                tags.join(",")
            );

            println!("Successfully generated rule: {}", query);

            let route = "/services/search/jobs".to_string();
            let resp = reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap()
                .post(self.splunk.to_owned() + &route)
                .header("Authorization", Credentials::new(&self.username, &self.password).as_http_header())
                .form(&[("search", query)])
                .send()?
                .text()?;
            let start_sid = resp.find("<sid>").unwrap_or(0) + 5;
            let end_sid = resp.find("</sid>").unwrap_or(resp.len());
            let sid = &String::from(resp)[start_sid..end_sid];
            

            let check_route = "/services/search/jobs/".to_string() + &sid;
            loop {
                thread::sleep(Duration::from_millis(2000));
                println!("Asking for state");
                let resp = reqwest::blocking::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .unwrap()
                    .get(self.splunk.to_owned() + &check_route)
                    .header("Authorization", Credentials::new("analyst", "analyst!").as_http_header())
                    .send()?
                    .text()?;

                let start_state = resp.find("dispatchState").unwrap_or(0) + 15;
                let state = &String::from(resp)[start_state..];

                println!("ask 4 state");
                if state.starts_with("FINALIZING") || state.starts_with("FAILED") || state.starts_with("DONE") {
                    println!("Done with {:?}", file);
                    break;
                }

            }

        }
        Ok(())
    }

    pub fn run_many_queries(&self) -> Result<()> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.threads + 1)
            .build()
            .unwrap();
    
        let (tx, rx) = std::sync::mpsc::channel();
        pool.scope(move |s| {
            for file in WalkDir::new(&self.rules).into_iter().filter_map(|file| file.ok()) {
                if file.metadata().unwrap().is_file() {
                    let tx = tx.clone();
                    s.spawn(move |_| {
                        tx
                            .send(self.run_query(file.path()).unwrap())
                            .expect("Unable to send task");
                    });
                }
            }
            drop(tx);
            let _res: Vec<()> = rx.into_iter().collect();
        });
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        if self.rules.is_file() {
            println!("Running a single rule from {:?}", self.rules);
            self.run_query(&self.rules)?;
        } else if self.rules.is_dir() {
            println!("Running all rules from {:?}", self.rules);
            self.run_many_queries()?;
        }
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
    let matches = clap::Command::new("Sigma 2 Splunk")
        .setting(AppSettings::DeriveDisplayOrder)
        .author("ljk")
        .about("Run sigma queries against a splunk instance.")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::new("RULES").required(true))
        .arg(
            Arg::new("splunk")
                .long("splunk")
                .short('s')
                .takes_value(true)
                .help("Splunk management url (eg: https://splunk.fak:8089)")
                .required(true),
        )
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
        .arg(
            Arg::new("earliest")
                .long("earliest")
                .short('e')
                .takes_value(true)
                .default_value("1y")
                .help("Add `earliest=` to your searches"),
        )
        .get_matches();

        Sigma2Splunk::from_matches(&matches)?.run()?;
        Ok(())
}