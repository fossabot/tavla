//! _tavla_ is a tiny rust crate for simple speech
//! synthesis tasks. It opens external processes
//! to speak strings out loud with minimal setup.
//! It uses system-provided speech synthesis on Mac
//! and Windows, and can also use `espeak` (on any system).
//!
//! Using it is as simple as this:
//! ```
//! extern crate tavla;
//!
//! use tavla::{Voice, Speech, any_voice};
//!
//! # fn main() -> Result<(), tavla::Error> {
//! any_voice()?
//! # // make espeak silent for the test
//! # // let mut voice = any_voice()?; voice.mute(); voice
//!     .speak("Oh _my_, the computer is _talking_!")?
//!     .await_done()?;
//!
//! let voice = any_voice();
//! let speech = any_voice()?
//!     .speak(".Isn't that.. _fascinating_?")?;
//! assert!(!speech.is_done()?);
//! speech.await_done()?;
//!
//! any_voice()?
//!     .speak("I have some doubts about giving \
//!             computers the power of speech!")?
//!     .cancel()?; // Nonsense... Sh sh shhh...
//! # Ok(())
//! # }
//! ```

extern crate failure;
#[cfg(test)]
extern crate tempfile;

mod any;
mod child;
#[cfg(target_os = "windows")]
mod cscript;
mod espeak;
mod factory;
mod prelude;
#[cfg(target_os = "macos")]
mod say;
mod speech;
mod token;
mod version;
mod voice;

pub use prelude::*;
