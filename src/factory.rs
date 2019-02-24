use crate::any::AnyVoice;
use crate::espeak::{Error as EspeakError, Espeak};
use crate::say::{Error as SayError, Say};
use crate::cscript::{Error as CScriptVoiceError, CScriptVoice};
use failure::{bail, Error};

/// Picks any available voice and wraps it in
/// [`AnyVoice`](enum.AnyVoice.html).
///
/// It prefers `espeak`, if installed and then
/// tries for system-provided speech synthesis.
pub fn any_voice() -> Result<AnyVoice, Error> {
    if let Ok(espeak) = espeak() {
        Ok(espeak.into())
    } else if let Ok(say) = say() {
        Ok(say.into())
    } else if let Ok(cscript) = CScriptVoice::new() {
        Ok(cscript.into())
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

/// Tries to initialize a [`Say`](struct.Say.html) voice,
/// commonly available on Mac systems.
///
/// This likely will fail on non-Mac systems, unless a
/// command line tool with the same name and semantics
/// is found.
pub fn say() -> Result<Say, SayError> {
    Say::new()
}

/// Tries to initialize a voice for Windows systems that
/// works by generating a VB script for reading out loud
/// text from stdin and then running it with `cscript`,
/// piping text into it.
///
/// Works on most Windows systems and may also work with
/// some setups with `Wine` or similar compatibility layers.
pub fn cscript_voice() -> Result<CScriptVoice, CScriptVoiceError> {
    CScriptVoice::new()
}
