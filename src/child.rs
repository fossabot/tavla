pub use err::Error;
use std::process::{Child, ExitStatus};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::Duration;
use std::thread::sleep;

/// Time between checks in `await_done`
const AWAIT_DONE_CHECK_INTERVAL : Duration = Duration::from_millis(5);
/// Time that a child process has for graceful exit before being forcibly
/// killed.
const CANCEL_GRACE_PERIOD : Duration = Duration::from_millis(25);

/// Ongoing or finished [`Speech`](trait.Speech.html) in an external process.
pub struct Speech {
    state: Mutex<State>
}

impl Speech {
    pub fn new(mut child: Child) -> Self {
        let mut state = State::Running(child);
        state.update();

        Speech {
            state: Mutex::new(state)
        }
    }
}

impl crate::Speech for Speech {
    type Error = Error;

    /// Waits until the speech is finished or has been
    /// cancelled and returns `Ok(())`.
    ///
    /// If already exited successfully before the call,
    /// or has been manually cancelled also reports `Ok(())`.
    ///
    /// Returns error on I/O errors during checking or
    /// when an unsuccessful exit status has been reported,
    /// except is has been cancelled.
    fn await_done(&self) -> Result<(), Self::Error> {
        loop {
            if self.is_done()? {
                return Ok(());
            }

            // Unlock most of the time so `is_done` calls
            // do not have to wait too long for an `await_done`.

            // FIXME we should let the OS wake up as soon as possible,
            // rather than always waiting the full check interval
            sleep(AWAIT_DONE_CHECK_INTERVAL);
        }
    }

    /// Checks if the speech is over, either because it
    /// finished by itself, or because it was cancelled.
    /// 
    /// Returns an error on unsuccessful exit status.
    fn is_done(&self) -> Result<bool, Self::Error> {
        let mut state = self.state.try_lock()
            .expect("Failed to obtain lock on child process");

        state.update();
        state.exited_successfully()
    }

    /// Cancels the ongoing speech. Can safely be called
    /// after the speech has finished or cancelled.
    /// `await_done` will report an unsuccessful exit
    /// error if called after `cancel`.
    fn cancel(&mut self) -> Result<(), Self::Error> {
        let mut state = self.state.try_lock()
            .expect("Failed to obtain lock on child process");

        state.cancel()
    }
}

enum State {
    Running(Child),
    Done(ExitStatus),
    Cancelled
}

impl State {
    fn close(&mut self, status: ExitStatus) {
        *self = State::Done(status);
    }

    fn update(&mut self) {
        match self {
            State::Running(child) => {
                let status = child.try_wait()
                    .expect("Failed to obtain check if child process has exited");

                if let Some(status) = status {
                    *self = State::Done(status);
                }
            },
            _ => (), // Done state and cancelled are both terminal, no need to update
        }
    }

    fn exited_successfully(&self) -> Result<bool, Error> {
        match self {
            State::Running(_) => Ok(false),
            State::Done(status) => if status.success() {
                Ok(true)
            } else {
                Err(Error::exit_failure(status.clone()))
            },
            State::Cancelled => Ok(true)
        }
    }

    fn cancel(&mut self) -> Result<(), Error> {
        self.update();
        if let State::Running(child) = self {
            child.kill()
                .expect("Failed to send termination signal to child");

            sleep(CANCEL_GRACE_PERIOD);
        }

        // Check if cancellation worked
        self.update();
        match self {
            State::Running(_) => Err(Error::cancel_ignored()),
            State::Cancelled => Ok(()), // Another thread must have cancelled
            State::Done(_) => {
                *self = State::Cancelled;
                Ok(())
            }
        }
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
            display = "attempted to cancel child process, but is still running"
        )]
        CancelIgnored { backtrace: Backtrace },
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

        pub fn cancel_ignored() -> Self {
            Error::CancelIgnored {
                backtrace: Backtrace::new(),
            }
        }
    }
}
