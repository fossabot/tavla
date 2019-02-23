use err::Error;
use std::process::Child;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

/// Ongoing or finished [`Speech`](trait.Speech.html) in an external process.
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
    type Error = Error;

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

mod err {
    use failure::{Backtrace, Fail};
    use std::io;
    use std::process::ExitStatus;

    /// Errors during interaction with a speech synthesizer in an
    /// external process.
    #[derive(Fail, Debug)]
    pub enum Error {
        #[fail(display = "error while speech was running: {}", cause)]
        CannotAwait {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
        #[fail(display = "speech reported unsuccessful exit: {}", status)]
        ExitFailure {
            status: ExitStatus,
            backtrace: Backtrace,
        },
        #[fail(display = "attempt to terminate speech failed")]
        CannotCancel {
            #[cause]
            cause: io::Error,
            backtrace: Backtrace,
        },
        #[fail(
            display = "failed to cancel speech since another thread is trying to await the end of the speech"
        )]
        CancelConflict { backtrace: Backtrace },
    }

    impl Error {
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
