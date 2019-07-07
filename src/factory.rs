use crate::any::AnyVoice;
#[cfg(target_os = "windows")]
use crate::cscript::{CScriptVoice, Error as CScriptVoiceError};
use crate::espeak::{Error as EspeakError, Espeak};
#[cfg(target_os = "macos")]
use crate::say::{Error as SayError, Say};
use failure::{bail, Error};

/// Picks any available voice and wraps it in
/// [`AnyVoice`](enum.AnyVoice.html).
///
/// It prefers `espeak`, if installed and then
/// tries for system-provided speech synthesis.
pub fn any_voice() -> Result<AnyVoice, Error> {
    // Try espeak first, it is the only one that can provide
    // a consistent experience on different platforms.
    if let Ok(espeak) = espeak() {
        return Ok(espeak.into());
    }

    // When no espeak, try the built-in `say` command on mac.
    #[cfg(target_os = "macos")]
    {
        if let Ok(say) = say() {
            return Ok(say.into());
        }
    }

    // On windows, work with cscript.exe, if no espeak available
    #[cfg(target_os = "windows")]
    {
        if let Ok(cscript) = CScriptVoice::new() {
            return Ok(cscript.into());
        }
    }

    bail!("No pre-installed voice found")
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
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "windows")]
pub fn cscript_voice() -> Result<CScriptVoice, CScriptVoiceError> {
    CScriptVoice::new()
}
