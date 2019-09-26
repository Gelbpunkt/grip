extern crate clap;
extern crate colored;
extern crate regex;
extern crate walkdir;

use clap::{App, Arg};
use colored::*;
use regex::{Regex, RegexSet};
use std::fs::metadata;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::process;
use walkdir::WalkDir;

fn grip_file(path: &str, regex_set: &RegexSet, regexes: &Vec<Regex>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for (lineno, l) in reader.lines().enumerate() {
        let line: String;
        match l {
            Ok(v) => line = v,
            Err(_v) => return,
        }
        let matches: Vec<_> = regex_set.matches(&line).into_iter().collect();
        if matches.len() > 0 {
            for m in matches {
                let mtch = regexes[m].find(&line).unwrap().as_str();
                println!(
                    "{} {}: {}",
                    path.green(),
                    format!("L{}", lineno + 1).yellow(),
                    line.replace(mtch, &mtch.red().to_string())
                );
            }
        }
    }
}

fn grip_dir(path: &str, regex_set: RegexSet, regexes: Vec<Regex>) {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            grip_file(entry.path().to_str().unwrap(), &regex_set, &regexes);
        }
    }
}

fn grip(path: &str, md: std::fs::Metadata, regexes: &Vec<&str>) {
    let mut raw_regexes = Vec::new();

    for re in regexes {
        match Regex::new(re) {
            Ok(v) => raw_regexes.push(v),
            Err(_v) => {
                println!("Regex \"{}\" invalid.", re);
                process::exit(1)
            }
        }
    }

    let regex_set = RegexSet::new(regexes).unwrap();

    if md.is_file() {
        grip_file(path, &regex_set, &raw_regexes);
    } else {
        grip_dir(path, regex_set, raw_regexes);
    }
}

fn main() {
    let matches = App::new("grip")
        .version("1.0.0")
        .author("Jens Reidel <jens@troet.org>")
        .about("File content search for humans")
        .arg(
            Arg::with_name("match")
                .index(1)
                .takes_value(true)
                .multiple(false)
                .required_if("matches", ""),
        )
        .arg(
            Arg::with_name("matches")
                .help("the regexes to match")
                .takes_value(true)
                .short("-m")
                .long("--match")
                .multiple(true)
                .number_of_values(1)
                .required(false)
                .default_value("")
                .hide_default_value(true),
        )
        .arg(
            Arg::with_name("target")
                .help("the target to search in")
                .index(2)
                .takes_value(true)
                .multiple(true)
                .required(false)
                .default_value("."),
        )
        .get_matches();

    let mut regexes = Vec::new();
    let mut target = ""; // fix possibly uninit

    if let Some(reg) = matches.value_of("match") {
        regexes.push(reg);
    }

    if let Some(reg) = matches.values_of("matches") {
        for regex in reg {
            if regex != "" {
                regexes.push(regex);
            }
        }
    }

    if let Some(tgt) = matches.value_of("target") {
        target = tgt;
    }

    // file or directory?
    match metadata(target) {
        Ok(v) => grip(target, v, &regexes),
        Err(_v) => println!("Path \"{}\" not found.", target),
    };
}
