//! Speech synthesis with the `say` command line
//! tool available on mac systems.

pub use crate::child::Speech;
pub use err::Error;

use crate::token::{PauseDuration::*, Token, Tokenizer};
use crate::version::detect_version_with_arg;
use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

#[derive(Debug)]
pub struct Say;

/// A [`Voice`](trait.Voice.html) that works by opening
/// a shell and piping text into `say`.
impl Say {
    pub fn new() -> Result<Say, Error> {
        detect_version_with_arg("say", Some(""))
            .map(|_| Say)
            .map_err(Error::say_not_installed)
    }

    fn spawn(&self, output_file: Option<&Path>) -> Result<Child, Error> {
        let mut cmd = Command::new("say");

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null()) // Ignore standard output
            .stderr(Stdio::null()); // And error too

        if let Some(output) = output_file {
            cmd.arg("--data-format=LEF32@22050");
            cmd.arg("-o");
            cmd.arg(output);
        }

        cmd.spawn().map_err(Error::cannot_invoke)
    }

    fn speak(&self, sentence: &str, output_file: Option<&Path>) -> Result<Speech, Error> {
        let mut say = self.spawn(output_file)?;
        let pipe = say.stdin.take().ok_or_else(Error::cannot_open_pipe)?;

        self.write_say_markup(sentence, pipe)?;

        Ok(Speech::new(say))
    }

    fn write_say_markup(&self, sentence: &str, mut pipe: ChildStdin) -> Result<(), Error> {
        for token in Tokenizer::new(sentence) {
            match token {
                Token::Normal(text) => write!(pipe, "{}", text).map_err(Error::cannot_write)?,
                Token::Emphasised(text) => {
                    write!(pipe, "[[emph +]]{}[[emph -]]", text).map_err(Error::cannot_write)?
                }
                Token::Pause(Sentence) => {
                    write!(pipe, "[[slnc 350]]").map_err(Error::cannot_write)?
                }
                Token::Pause(Paragraph) => {
                    write!(pipe, "[[slnc 700]]").map_err(Error::cannot_write)?
                }
                Token::Pause(Seconds(secs)) => {
                    write!(pipe, "[[slnc {}000]]", secs).map_err(Error::cannot_write)?
                }
            }
        }
        writeln!(pipe, "").map_err(Error::cannot_write)?;
        pipe.flush().map_err(Error::cannot_write)
    }
}

impl crate::Voice for Say {
    type Speech = Speech;
    type Error = Error;

    /// Speaks the given sentence. Emphasized words can be wrapped in underscores.
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
    use std::io::Error as IoError;

    #[derive(Debug, Fail)]
    pub enum Error {
        #[fail(display = "say command could not be found: {}", _0)]
        SayNotInstalled(#[cause] VersionDetectError),
        #[fail(display = "say command could not be invoked: {}", cause)]
        CannotInvoke {
            #[cause]
            cause: IoError,
            backtrace: Backtrace,
        },
        #[fail(display = "say command could not be written to: {}", cause)]
        CannotWrite {
            #[cause]
            cause: IoError,
            backtrace: Backtrace,
        },
        #[fail(display = "cannot open pipe to say")]
        CannotOpenPipe(Backtrace),
    }

    impl Error {
        pub fn say_not_installed(cause: VersionDetectError) -> Self {
            Error::SayNotInstalled(cause)
        }

        pub fn cannot_invoke(cause: IoError) -> Self {
            Error::CannotInvoke {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_write(cause: IoError) -> Self {
            Error::CannotWrite {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_open_pipe() -> Self {
            Error::CannotOpenPipe(Backtrace::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use crate::voice::Voice;
    use crate::speech::Speech;
    use tempfile::tempdir;

    #[test]
    fn say_to_file() {
        // given
        let say = Say::new().unwrap();
        let tempdir = tempdir().expect("could not make temporary directory for test");
        let target_path = tempdir.path().join("testsay.wav");

        // when
        say.speak_to_file("This is a test sentence to speak.", &target_path)
            .expect("Failed to start speaking to file")
            .await_done()
            .expect("Failed to wait until speaking to file is done");

        let generated_file_meta = File::open(&target_path)
            .expect("could not open generated file")
            .metadata()
            .expect("could not obtain metadata of generated file");

        // then
        assert!(target_path.exists(), "Expecting speakint to path to produce a file.");
        assert!(
            generated_file_meta.len() > 1024,
            "Expected test sentence to add up to more than a KiB worth of WAV."
        );
    }
}
