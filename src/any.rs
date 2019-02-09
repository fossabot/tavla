use crate::{Espeak, EspeakSpeech, Speech, Voice};
use failure::Error;

/// A [`Voice`](trait.Voice.html) that works with any of
/// the built-in techniques (currently only espeak).
pub enum AnyVoice {
    Espeak(Espeak),
}

/// A [`Speech`](trait.Speech.html) with any built-in
/// backend.
pub enum AnySpeech {
    Espeak(EspeakSpeech),
}

impl From<Espeak> for AnyVoice {
    fn from(espeak: Espeak) -> Self {
        AnyVoice::Espeak(espeak)
    }
}

impl Voice for AnyVoice {
    type Error = Error;
    type Speech = AnySpeech;

    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        match self {
            AnyVoice::Espeak(voice) => Ok(AnySpeech::Espeak(voice.speak(sentence)?)),
        }
    }
}

impl Speech for AnySpeech {
    type Error = Error;

    fn await_done(&self) -> Result<(), Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.await_done()?),
        }
    }

    fn is_done(&self) -> Result<bool, Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.is_done()?),
        }
    }

    fn cancel(&self) -> Result<(), Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.cancel()?),
        }
    }
}
