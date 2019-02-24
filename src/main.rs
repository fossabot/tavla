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

use std::io::{stdin, BufRead};
use clap::{App, Arg};
use failure::bail;

fn main() -> Result<(), failure::Error> {
    let args = App::new("tavla")
        .version(env!("CARGO_PKG_VERSION"))
        .author("krachzack <hello@phstadler.com>")
        .about("Speaks text from arguments or from stdin")
        .arg(Arg::with_name("stdin")
            .short("i")
            .long("stdin")
            .help("Read input from stdin instead of command line args"))
        .arg(Arg::with_name("INPUT")
            .help("Other args are spoken aloud")
            .multiple(true)
            .takes_value(true))
        .get_matches();

    let voice = any_voice()?;
    if args.is_present("stdin") {
        for line in stdin().lock().lines() {
            voice.speak(line?)?
                .await_done()?;
        }
    } else {
        match args.values_of("INPUT") {
            Some(input_args) => voice.speak(join(input_args))?
                    .await_done()?,
            None => bail!("No command line arguments for speech specified")
        }
    }

    Ok(())
}

fn join<'a, I>(iterator: I) -> String
    where I : IntoIterator<Item = &'a str> {
    
    let mut iter = iterator.into_iter();

    match iter.next() {
        Some(first) => iter.fold(
            String::from(first),
            |mut acc, next| {
                acc.push(' ');
                acc.push_str(next);
                acc
            }
        ),
        None => String::new()
    }
}
