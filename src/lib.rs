//! _tavla_ is a tiny rust crate for simple speech
//! synthesis tasks. It opens external processes
//! to speak strings out loud with minimal setup.
//! It only supports espeak right now but may gain
//! additional backends in the future so it works
//! out of the box on Windows and Mac as well, using
//! system-provided speech synthesis.
//!
//! Using it is as simple as this:
//! ```
//! extern crate tavla;
//!
//! use tavla::{Voice, Speech, any_voice};
//! use std::thread::sleep;
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), tavla::Error> {
//! let speech = any_voice()?
//!     .speak("Oh _my_, the computer is _talking_!")?;
//! #  // make espeak silent for the test
//! # speech.cancel()?; let speech = any_voice()?.speak("")?;
//! speech.await_done()?;
//! assert!(speech.is_done()?);
//!
//! sleep(Duration::from_millis(1000));
//!
//! let speech = any_voice()?
//!     .speak("Isn't that.. _fascinating_?")?;
//! assert!(!speech.is_done()?);
//! #  // make espeak silent for the test
//! # speech.cancel()?; let speech = any_voice()?.speak("")?;
//! speech.await_done()?;
//!
//! let speech = any_voice()?
//!     .speak("I have some doubts about giving computers the power of speech!")?;
//! assert!(!speech.is_done()?);
//! speech.cancel()?; // Nonsense... Sh sh shhh...
//! assert!(speech.is_done()?);
//!
//! # Ok(())
//! # }
//! ```

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
