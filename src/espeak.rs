//! Speech synthesis with the `espeak` command line tool
//! commonly available on some linux and unix systems.
//! On Windows systems it would be rather exotic but
//! might be available.

pub use crate::child::Speech;
pub use err::Error;

use crate::token::{PauseDuration::*, Token, Tokenizer};
use crate::version::detect_version;
use crate::Voice;
use std::io::Write;
use std::process::{Child, Command, Stdio};

/// A [`Voice`](trait.Voice.html) that works by opening
/// a shell and piping text into `espeak`.
#[derive(Debug)]
pub struct Espeak;

impl Espeak {
    pub fn new() -> Result<Espeak, Error> {
        detect_version("espeak").map_err(Error::espeak_not_installed)?;
        Ok(Espeak)
    }

    fn open_espeak(&self) -> Result<Child, Error> {
        self.invoke(Command::new("espeak").args(&["--stdin", "-m"]))
    }

    fn invoke(&self, cmd: &mut Command) -> Result<Child, Error> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null()) // Ignore standard output
            .stderr(Stdio::null()) // And error too
            .spawn()
            .map_err(Error::cannot_invoke)
    }
}

impl Voice for Espeak {
    type Speech = Speech;
    type Error = Error;

    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        let mut espeak = self.open_espeak()?;
        let pipe = espeak.stdin.take();
        let mut pipe = pipe.ok_or_else(Error::cannot_open_pipe)?;
        write!(pipe, "<speak>").map_err(Error::cannot_write)?;
        for token in Tokenizer::new(sentence.as_ref()) {
            match token {
                Token::Normal(text) => write!(pipe, "{}", text).map_err(Error::cannot_write)?,
                Token::Emphasised(text) => {
                    write!(pipe, "<emphasis>{}</emphasis>", text).map_err(Error::cannot_write)?
                }
                Token::Pause(Sentence) => {
                    write!(pipe, "<break strength=\"medium\"/>").map_err(Error::cannot_write)?
                }
                Token::Pause(Paragraph) => {
                    write!(pipe, "<break strength=\"x-strong\"/>").map_err(Error::cannot_write)?
                }
                Token::Pause(Seconds(secs)) => {
                    write!(pipe, "<break time=\"{}s\"/>", secs).map_err(Error::cannot_write)?
                }
            }
        }
        writeln!(pipe, "</speak>").map_err(Error::cannot_write)?;
        pipe.flush().map_err(Error::cannot_write)?;
        Ok(Speech::new(espeak))
    }
}

mod err {
    use crate::version::Error as VersionDetectError;
    use failure::{Backtrace, Fail};
    use std::io;

    /// `Espeak`-specific errors during any phase of IO.
    #[derive(Fail, Debug)]
    pub enum Error {
        /// No `espeak` on path.
        #[fail(display = "espeak executable could not be found: {}", _0)]
        EspeakNotInstalled(#[cause] VersionDetectError),
        #[fail(display = "espeak could not be started: {}", cause)]
        CannotInvoke {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
        #[fail(display = "pipe to espeak could not be opened")]
        CannotOpenPipe { backtrace: Backtrace },
        #[fail(display = "pipe to espeak cannot be written: {}", cause)]
        CannotWrite {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
    }

    impl Error {
        pub fn espeak_not_installed(cause: VersionDetectError) -> Self {
            Error::EspeakNotInstalled(cause)
        }

        pub fn cannot_invoke(cause: io::Error) -> Self {
            Error::CannotInvoke {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_open_pipe() -> Self {
            Error::CannotOpenPipe {
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_write(cause: io::Error) -> Self {
            Error::CannotWrite {
                cause,
                backtrace: Backtrace::new(),
            }
        }
    }
}
