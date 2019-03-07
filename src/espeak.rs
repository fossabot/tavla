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
pub struct Espeak {
    espeak_command: String,
}

impl Espeak {
    pub fn new() -> Result<Espeak, Error> {
        detect_version("espeak").map_err(Error::espeak_not_installed)?;
        // TODO the piping should be doable without an external shell
        detect_version("sh").map_err(Error::sh_not_installed)?;

        Ok(Espeak {
            espeak_command: Self::espeak_command(None),
        })
    }

    /// Espeak subcommand for execution in a shell
    fn espeak_command(voice_name: Option<&str>) -> String {
        // -m enables SSML markup for <emphasis>
        let mut espeak_command = String::from("espeak --stdin -m");

        if let Some(voice) = voice_name.as_ref() {
            espeak_command.push_str(" -v ");
            espeak_command.push_str(voice);
        }

        let paplay_installed = detect_version("paplay").is_ok();
        if paplay_installed {
            espeak_command.push_str(" --stdout | paplay");
        }

        espeak_command
    }

    fn open_espeak(&self) -> Result<Child, Error> {
        self.invoke(Command::new("/bin/sh").args(&["-c", &self.espeak_command]))
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
                Token::Emphasised(text) => write!(pipe, "<emphasis>{}</emphasis>", text)
                    .map_err(Error::cannot_write)?,
                Token::Pause(Sentence) => write!(pipe, "<break strength=\"medium\"/>").map_err(Error::cannot_write)?,
                Token::Pause(Paragraph) => write!(pipe, "<break strength=\"x-strong\"/>")
                    .map_err(Error::cannot_write)?,
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
        #[fail(
            display = "No shell executable (sh) found, which is required to run espeak with tavla: {}",
            _0
        )]
        /// No `sh` shell on path.
        ShNotInstalled(#[cause] VersionDetectError),
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

        pub fn sh_not_installed(cause: VersionDetectError) -> Self {
            Error::ShNotInstalled(cause)
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
