use crate::speech::Speech;

/// A trait for things that can speak.
pub trait Voice {
    type Speech: Speech;
    type Error;

    /// Speaks the given sentence. Emphasized words can be wrapped in underscores.
    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>;
}
