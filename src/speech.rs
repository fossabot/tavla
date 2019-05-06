use failure::Fail;
use std::fmt::{Debug, Display};

/// An ongoing or finished piece of speech.
///
/// A speech is typically running asynchronously
/// and its end can be awaited using this trait.
///
/// Speeches can be aloud or to file.
pub trait Speech {
    type Error: Fail + Send + Sync + Debug + Display;

    /// Waits until the speech is finished.
    fn await_done(&self) -> Result<(), Self::Error>;

    /// Checks if the speech is over.
    fn is_done(&self) -> Result<bool, Self::Error>;

    /// Ends the speech, if still running, otherwise no effect.
    fn cancel(&mut self) -> Result<(), Self::Error>;
}
