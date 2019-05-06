#![feature(const_str_as_bytes)]

extern crate clap;
extern crate failure;

mod any;
mod child;
#[cfg(windows)]
mod cscript;
mod espeak;
mod factory;
mod prelude;
mod say;
mod speech;
mod token;
mod version;
mod voice;

pub use prelude::*;

use clap::{App, Arg};
use failure::bail;
use std::io::{stdin, BufRead, Read};
use std::path::Path;

fn main() -> Result<(), failure::Error> {
    let args = App::new("tavla")
        .version(env!("CARGO_PKG_VERSION"))
        .author("krachzack <hello@phstadler.com>")
        .about("Speaks text from arguments or from stdin")
        .arg(
            Arg::with_name("stdin")
                .short("i")
                .long("stdin")
                .help("Read input from stdin instead of command line args"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("Write a WAV file to the specified path instead of speaking out loud")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Other args are spoken aloud")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    let voice = any_voice()?;
    let target_file = args.value_of("file").map(Path::new);

    if args.is_present("stdin") {
        if let Some(target_file) = target_file {
            let mut text = String::new();
            stdin().lock().read_to_string(&mut text)?;
            voice.speak_to_file(text, target_file)?.await_done()?;
        } else {
            for line in stdin().lock().lines() {
                voice.speak(line?)?.await_done()?;
            }
        }
    } else {
        match args.values_of("INPUT") {
            Some(input_args) => {
                let text = join(input_args);
                match target_file {
                    None => voice.speak(text),
                    Some(target_file) => voice.speak_to_file(text, target_file),
                }?
                .await_done()?
            }
            None => bail!("No command line arguments for speech specified"),
        }
    }

    Ok(())
}

fn join<'a, I>(iterator: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let mut iter = iterator.into_iter();

    match iter.next() {
        Some(first) => iter.fold(String::from(first), |mut acc, next| {
            acc.push(' ');
            acc.push_str(next);
            acc
        }),
        None => String::new(),
    }
}
