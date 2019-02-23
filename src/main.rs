extern crate failure;

mod any;
mod espeak;
mod factory;
mod prelude;
mod speech;
mod token;
mod version;
mod voice;

pub use prelude::*;

use std::env::args;

fn main() -> Result<(), failure::Error> {
    any_voice()?.speak(args_text())?;

    Ok(())
}

fn args_text() -> String {
    args()
        .skip(1) // disregard program name
        .fold(String::new(), |mut text, next| {
            text.push_str(&next);
            text.push(' ');
            text
        })
}
