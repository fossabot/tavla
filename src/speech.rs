/// An ongoing or finished piece of speech.
///
/// A speech is typically running asynchronously
/// and its end can be awaited using this trait.
pub trait Speech {
    type Error;

    /// Waits until the speech is finished.
    fn await_done(&self) -> Result<(), Self::Error>;

    /// Checks if the speech is over.
    fn is_done(&self) -> Result<bool, Self::Error>;

    /// Ends the speech, if still running, otherwise no effect.
    fn cancel(&self) -> Result<(), Self::Error>;
}
