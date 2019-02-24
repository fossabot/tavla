#[cfg(windows)]
use crate::{CScriptVoice, CScriptVoiceSpeech};
use crate::{Espeak, EspeakSpeech, Say, SaySpeech};
use failure::Error;

/// A [`Voice`](trait.Voice.html) that works with any of
/// the built-in techniques (currently only espeak).
pub enum AnyVoice {
    #[cfg(windows)]
    CScript(CScriptVoice),
    Espeak(Espeak),
    Say(Say),
}

/// A [`Speech`](trait.Speech.html) with any built-in
/// backend.
pub enum AnySpeech {
    #[cfg(windows)]
    CScript(CScriptVoiceSpeech),
    Espeak(EspeakSpeech),
    Say(SaySpeech),
}

#[cfg(windows)]
impl From<CScriptVoice> for AnyVoice {
    fn from(cscript: CScriptVoice) -> Self {
        AnyVoice::CScript(cscript)
    }
}

impl From<Espeak> for AnyVoice {
    fn from(espeak: Espeak) -> Self {
        AnyVoice::Espeak(espeak)
    }
}

impl From<Say> for AnyVoice {
    fn from(say: Say) -> Self {
        AnyVoice::Say(say)
    }
}

impl crate::Voice for AnyVoice {
    type Error = Error;
    type Speech = AnySpeech;

    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        match self {
            #[cfg(windows)]
            AnyVoice::CScript(voice) => Ok(AnySpeech::CScript(voice.speak(sentence)?)),
            AnyVoice::Espeak(voice) => Ok(AnySpeech::Espeak(voice.speak(sentence)?)),
            AnyVoice::Say(voice) => Ok(AnySpeech::Say(voice.speak(sentence)?)),
        }
    }
}

impl crate::Speech for AnySpeech {
    type Error = Error;

    fn await_done(&self) -> Result<(), Error> {
        match self {
            #[cfg(windows)]
            AnySpeech::CScript(speech) => Ok(speech.await_done()?),
            AnySpeech::Espeak(speech) => Ok(speech.await_done()?),
            AnySpeech::Say(speech) => Ok(speech.await_done()?),
        }
    }

    fn is_done(&self) -> Result<bool, Error> {
        match self {
            #[cfg(windows)]
            AnySpeech::CScript(speech) => Ok(speech.is_done()?),
            AnySpeech::Espeak(speech) => Ok(speech.is_done()?),
            AnySpeech::Say(speech) => Ok(speech.is_done()?),
        }
    }

    fn cancel(&self) -> Result<(), Error> {
        match self {
            #[cfg(windows)]
            AnySpeech::CScript(speech) => Ok(speech.cancel()?),
            AnySpeech::Espeak(speech) => Ok(speech.cancel()?),
            AnySpeech::Say(speech) => Ok(speech.cancel()?),
        }
    }
}
