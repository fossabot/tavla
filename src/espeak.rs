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
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

/// A [`Voice`](trait.Voice.html) that works by opening
/// a shell and piping text into `espeak`.
#[derive(Debug)]
pub struct Espeak;

impl Espeak {
    pub fn new() -> Result<Espeak, Error> {
        detect_version("espeak").map_err(Error::espeak_not_installed)?;
        Ok(Espeak)
    }

    fn open_espeak(&self, output_wav_path: Option<&Path>) -> Result<Child, Error> {
        let mut cmd = Command::new("espeak");

        cmd.arg("-m");
        if let Some(output_wav) = output_wav_path {
            cmd.arg("-w");
            cmd.arg(output_wav);
        }

        self.invoke(&mut cmd)
    }

    fn invoke(&self, cmd: &mut Command) -> Result<Child, Error> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null()) // Ignore standard output
            .stderr(Stdio::null()) // And error too
            .spawn()
            .map_err(Error::cannot_invoke)
    }

    fn speak(&self, sentence: &str, output_wav_path: Option<&Path>) -> Result<Speech, Error> {
        let mut espeak = self.open_espeak(output_wav_path)?;
        espeak
            .stdin
            .take()
            .ok_or_else(Error::cannot_open_pipe)
            .and_then(|p| self.write_ssml_to_pipe(sentence.as_ref(), p))
            .map(|_| Speech::new(espeak))
    }

    fn write_ssml_to_pipe(&self, raw_text: &str, mut pipe: ChildStdin) -> Result<(), Error> {
        write!(pipe, "<speak>").map_err(Error::cannot_write)?;
        for token in Tokenizer::new(raw_text) {
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
        pipe.flush().map_err(Error::cannot_write)
    }
}

impl Voice for Espeak {
    type Speech = Speech;
    type Error = Error;

    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        self.speak(sentence.as_ref(), None)
    }

    fn speak_to_file<S, P>(
        &self,
        sentence: S,
        wav_file_path: P,
    ) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        self.speak(sentence.as_ref(), Some(wav_file_path.as_ref()))
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
