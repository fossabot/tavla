use crate::speech::Speech;
use std::fmt::{Debug, Display};
use failure::Fail;

/// A trait for things that can speak.
pub trait Voice {
    type Speech: Speech;
    type Error : Fail + Send + Sync + Debug + Display;

    /// Speaks the given sentence. Emphasized words can be wrapped in underscores.
    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>;
}
