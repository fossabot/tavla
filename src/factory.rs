use crate::any::AnyVoice;
use crate::espeak::{Error as EspeakError, Espeak};
use crate::say::{Error as SayError, Say};
use failure::{bail, Error};

/// Picks any available voice and wraps it in
/// [`AnyVoice`](enum.AnyVoice.html).
pub fn any_voice() -> Result<AnyVoice, Error> {
    if let Ok(espeak) = Espeak::new() {
        Ok(espeak.into())
    } else if let Ok(say) = Say::new() {
        Ok(say.into())
    } else {
        bail!("No pre-installed voice found")
    }
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

/// Tries to initialize an [`Espeak`](struct.Espeak.html)
/// voice.
///
/// Requires `espeak` and `sh` to be available on the
/// path. If `paplay` is available, it will be used
/// for output.
pub fn say() -> Result<Say, SayError> {
    Say::new()
}
