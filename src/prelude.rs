pub use crate::any::{AnySpeech, AnyVoice};
#[cfg(windows)]
pub use crate::cscript::{CScriptVoice, Error as CScriptVoiceError, Speech as CScriptVoiceSpeech};
pub use crate::espeak::{Error as EspeakError, Espeak, Speech as EspeakSpeech};
pub use crate::factory::*;
pub use crate::say::{Error as SayError, Say, Speech as SaySpeech};
pub use crate::speech::Speech;
pub use crate::voice::Voice;
pub use failure::Error;
