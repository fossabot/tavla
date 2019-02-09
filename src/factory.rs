use crate::any::AnyVoice;
use crate::espeak::{Error as EspeakError, Espeak};
use failure::Error;

/// Picks any available voice and wraps it in
/// [`AnyVoice`](enum.AnyVoice.html).
pub fn any_voice() -> Result<AnyVoice, Error> {
    Ok(From::from(Espeak::new()?))
}

/// Tries to initialize an [`Espeak`](struct.Espeak.html)
/// voice.
///
/// Requires `espeak` and `sh` to be available on the
/// path. If `paplay` is available, it will be used
/// for output.
pub fn espeak() -> Result<Espeak, EspeakError> {
    Espeak::new()
}
