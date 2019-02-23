//! Speech synthesis with the `say` command line
//! tool available on mac systems.

pub use err::Error;
pub use crate::child::Speech;

use std::io::Write;
use std::process::{Command, Child, Stdio};
use crate::version::detect_version_with_arg;
use crate::token::{Tokenizer, Token, PauseDuration::*};

#[derive(Debug)]
pub struct Say;

/// A [`Voice`](trait.Voice.html) that works by opening
/// a shell and piping text into `say`.
impl Say {
    pub fn new() -> Result<Say, Error> {
        detect_version_with_arg("say", "")
            .map(|_| Say)
            .map_err(Error::say_not_installed)
    }

    fn spawn(&self) -> Result<Child, Error> {
        Command::new("say")
            .stdin(Stdio::piped())
            .stdout(Stdio::null()) // Ignore standard output
            .stderr(Stdio::null()) // And error too
            .spawn()
            .map_err(Error::cannot_invoke)
    }
}

impl crate::Voice for Say {
    type Speech = Speech;
    type Error = Error;

    /// Speaks the given sentence. Emphasized words can be wrapped in underscores.
    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>
        {

        let mut say = self.spawn()?;
        let pipe = say.stdin.as_mut().ok_or_else(Error::cannot_open_pipe)?;

        for token in Tokenizer::new(sentence.as_ref()) {
            match token {
                Token::Normal(text) => write!(pipe, "{}", text).map_err(Error::cannot_write)?,
                Token::Emphasised(text) => write!(pipe, "[[emph +]]{}[[emph -]]", text)
                    .map_err(Error::cannot_write)?,
                Token::Pause(Sentence) => write!(pipe, "[[slnc 350]]").map_err(Error::cannot_write)?,
                Token::Pause(Paragraph) => write!(pipe, "[[slnc 700]]")
                    .map_err(Error::cannot_write)?,
                Token::Pause(Seconds(secs)) => {
                    write!(pipe, "[[slnc {}000]]", secs).map_err(Error::cannot_write)?
                }
            }
        }

        Ok(Speech::new(say))
    }
}

mod err {
    use failure::{Fail, Backtrace};
    use crate::version::Error as VersionDetectError;
    use std::io::Error as IoError;

    #[derive(Debug, Fail)]
    #[fail(display = "Too bad")]
    pub enum Error {
        #[fail(display = "say command could not be found: {}", _0)]
        SayNotInstalled(#[cause] VersionDetectError),
        #[fail(display = "say command could not be invoked: {}", cause)]
        CannotInvoke {
            #[cause] cause: IoError,
            backtrace: Backtrace
        },
        #[fail(display = "say command could not be written to: {}", cause)]
        CannotWrite {
            #[cause] cause: IoError,
            backtrace: Backtrace
        },
        #[fail(display = "cannot open pipe to say")]
        CannotOpenPipe(Backtrace)
    }

    impl Error {
        pub fn say_not_installed(cause: VersionDetectError) -> Self {
            Error::SayNotInstalled(cause)
        }

        pub fn cannot_invoke(cause: IoError) -> Self {
            Error::CannotInvoke {
                cause,
                backtrace: Backtrace::new()
            }
        }

        pub fn cannot_write(cause: IoError) -> Self {
            Error::CannotWrite {
                cause,
                backtrace: Backtrace::new()
            }
        }

        pub fn cannot_open_pipe() -> Self {
            Error::CannotOpenPipe(Backtrace::new())
        }
    }
}
