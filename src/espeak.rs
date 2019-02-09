pub use err::Error;
pub use speech::Speech;

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
        println!("Executing: {:?}", &self.espeak_command);
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
        {
            let pipe = &mut espeak.stdin;
            let pipe = pipe.as_mut().ok_or_else(Error::cannot_open_pipe)?;
            for token in Tokenizer::new(sentence.as_ref()) {
                match token {
                    Token::Normal(text) => write!(pipe, "{}", text).map_err(Error::cannot_write)?,
                    Token::Emphasised(text) => write!(pipe, "<emphasis>{}</emphasis>", text)
                        .map_err(Error::cannot_write)?,
                    Token::Pause(Sentence) => write!(pipe, ".").map_err(Error::cannot_write)?,
                    Token::Pause(Paragraph) => write!(pipe, "<break strength=\"x-strong\"/>")
                        .map_err(Error::cannot_write)?,
                    Token::Pause(Seconds(secs)) => {
                        write!(pipe, "<break time=\"{}s\"/>", secs).map_err(Error::cannot_write)?
                    }
                }
            }
        }
        Ok(Speech::new(espeak))
    }
}

mod speech {
    use super::Error;
    use std::process::Child;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    };

    /// Ongoing or finished [`Speech`](trait.Speech.html) originating
    /// from an espeak invocation.
    ///
    /// Designed only for single-threaded use.
    pub struct Speech {
        child: Mutex<Child>,
        done: AtomicBool,
    }

    impl Speech {
        pub fn new(mut espeak: Child) -> Self {
            let done = espeak
                .try_wait()
                .map(|status| status.is_some())
                .unwrap_or(false);

            Speech {
                child: Mutex::new(espeak),
                done: AtomicBool::new(done),
            }
        }
    }

    impl crate::Speech for Speech {
        type Error = super::Error;

        /// Waits until the speech is finished and returns
        /// `Ok(())`.
        ///
        /// If already exited successfully before the call,
        /// also reports `Ok(())`. If manually cancelled
        /// or otherwise exited with an unsuccessful exit status,
        /// returns `Err(ExitFailure)`.
        ///
        /// Returns error on I/O errors during checking.
        ///
        /// Also returns an `ExitFailure` error if cancelled
        /// with `cancel`.
        fn await_done(&self) -> Result<(), Self::Error> {
            self.child
                .lock()
                .expect("Failed to obtain lock on espeak child")
                .wait()
                .map_err(Error::cannot_await)
                .and_then(|status| {
                    self.done.store(true, Ordering::SeqCst);
                    if status.success() {
                        Ok(())
                    } else {
                        Err(Error::exit_failure(status))
                    }
                })
        }

        /// Checks if the speech is over, either because it
        /// finished by itself, or because it was cancelled.
        fn is_done(&self) -> Result<bool, Self::Error> {
            if self.done.load(Ordering::SeqCst) {
                Ok(true)
            } else {
                match self.child.try_lock() {
                    // Could obtain the lock, check if finished by now
                    Ok(mut child) => child.try_wait().map_err(Error::cannot_await).map(|status| {
                        if status.is_some() {
                            self.done.store(true, Ordering::SeqCst);
                            true
                        } else {
                            false
                        }
                    }),
                    // Probably someone else has the lock, awaiting the result
                    // assume not done (could also be another is_done call, which
                    // would be a false negative)
                    Err(_) => Ok(false),
                }
            }
        }

        /// Cancels the ongoing speech. This may fail if
        /// another thread is trying to await the end of
        /// the speech. Can safely be called after the
        /// speech has finished or cancelled.
        /// `await_done` will report an unsuccessful exit
        /// error if called after `cancel`.
        fn cancel(&self) -> Result<(), Self::Error> {
            let mut child = self
                .child
                .try_lock()
                .map_err(|_| Error::cancel_conflict())?;

            if !self.is_done()? {
                child.kill().map_err(Error::cannot_cancel)?;
                self.done.store(true, Ordering::SeqCst);
            }

            Ok(())
        }
    }
}

mod err {
    use crate::version::Error as VersionDetectError;
    use failure::{Backtrace, Fail};
    use std::io;
    use std::process::ExitStatus;

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
        #[fail(display = "error while espeak was running: {}", cause)]
        CannotAwait {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
        #[fail(display = "espeak reported unsuccessful exit: {}", status)]
        ExitFailure {
            status: ExitStatus,
            backtrace: Backtrace,
        },
        #[fail(display = "attempt to terminate espeak process failed")]
        CannotCancel {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
        #[fail(
            display = "failed to cancel espeak since another thread is trying to await the end of the speech"
        )]
        CancelConflict { backtrace: Backtrace },
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
        pub fn cannot_await(cause: io::Error) -> Self {
            Error::CannotAwait {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn exit_failure(status: ExitStatus) -> Self {
            Error::ExitFailure {
                status,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cannot_cancel(cause: io::Error) -> Self {
            Error::CannotCancel {
                cause,
                backtrace: Backtrace::new(),
            }
        }

        pub fn cancel_conflict() -> Self {
            Error::CancelConflict {
                backtrace: Backtrace::new(),
            }
        }
    }
}
