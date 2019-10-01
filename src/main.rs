extern crate atty;
extern crate clap;
extern crate colored;
extern crate regex;
extern crate walkdir;

use atty::Stream;
use clap::{App, Arg};
use colored::*;
use regex::{Regex, RegexSet};
use std::fs::metadata;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process;
use walkdir::WalkDir;

fn grip_line(
    line: String,
    regex_set: &RegexSet,
    regexes: &Vec<Regex>,
    use_color: bool,
    lineno: usize,
    path: &str,
) {
    let matches: Vec<_> = regex_set.matches(&line).into_iter().collect();
    if matches.len() > 0 {
        for m in matches {
            let mtch = regexes[m].find(&line).unwrap().as_str();
            if use_color {
                println!(
                    "{}{}: {}",
                    path.green(),
                    format!("L{}", lineno + 1).yellow(),
                    line.replace(mtch, &mtch.red().bold().to_string())
                );
            } else {
                println!("{}L{}: {}", path, lineno + 1, line);
            }
        }
    }
}

fn grip_file(path: &str, regex_set: &RegexSet, regexes: &Vec<Regex>, use_color: bool) {
    let file: File;
    match File::open(path) {
        Ok(v) => file = v,
        Err(_v) => {
            if use_color {
                return println!(
                    "{}",
                    format!("[WARN] Skipping {}, Permission Denied", path)
                        .red()
                        .bold()
                );
            } else {
                return println!("[WARN] Skipping {}, Permission Denied", path);
            }
        }
    }
    let reader = BufReader::new(file);

    let path = format!("{} ", path);

    for (lineno, l) in reader.lines().enumerate() {
        let line: String;
        match l {
            Ok(v) => line = v,
            Err(_v) => return,
        }
        grip_line(line, &regex_set, &regexes, use_color, lineno, &path);
    }
}

fn grip_dir(path: &str, regex_set: &RegexSet, regexes: &Vec<Regex>, use_color: bool) {
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            grip_file(
                entry.path().to_str().unwrap(),
                regex_set,
                regexes,
                use_color,
            );
        }
    }
}

fn grip(
    path: &str,
    md: std::fs::Metadata,
    regex_set: RegexSet,
    regexes: Vec<Regex>,
    use_color: bool,
) {
    if md.is_file() {
        grip_file(path, &regex_set, &regexes, use_color);
    } else {
        grip_dir(path, &regex_set, &regexes, use_color);
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
            Arg::with_name("no-color")
                .help("disable colored output")
                .short("-c")
                .long("--no-color"),
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

    let color = !matches.is_present("no-color");

    let mut raw_regexes = Vec::new();

    for re in &regexes {
        match Regex::new(re) {
            Ok(v) => raw_regexes.push(v),
            Err(_v) => {
                println!("Regex \"{}\" invalid.", re);
                process::exit(1)
            }
        }
    }

    let regex_set = RegexSet::new(regexes).unwrap();

    if !atty::is(Stream::Stdin) {
        let stdin = io::stdin();
        for (lineno, line) in stdin.lock().lines().enumerate() {
            let line = line.expect("Could not read line from standard in");
            grip_line(line, &regex_set, &raw_regexes, color, lineno, "");
        }

        return;
    }

    // file or directory?
    match metadata(target) {
        Ok(v) => grip(target, v, regex_set, raw_regexes, color),
        Err(_v) => println!("Path \"{}\" not found.", target),
    };
}
