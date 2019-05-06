use crate::speech::Speech;
use failure::Fail;
use std::fmt::{Debug, Display};
use std::path::Path;

/// A trait for things that can speak.
pub trait Voice {
    type Speech: Speech;
    type Error: Fail + Send + Sync + Debug + Display;

    /// Speaks the given sentence out loud.
    /// Emphasized words can be wrapped in underscores.
    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>;

    /// Speaks the given sentence to an uncompressed audio
    /// file in WAV format at the given path.
    fn speak_to_file<S, P>(
        &self,
        sentence: S,
        wav_file_path: P,
    ) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>;
}
