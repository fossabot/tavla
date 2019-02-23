use crate::{Espeak, EspeakSpeech, Say, SaySpeech};
use failure::Error;

/// A [`Voice`](trait.Voice.html) that works with any of
/// the built-in techniques (currently only espeak).
pub enum AnyVoice {
    Espeak(Espeak),
    Say(Say)
}

/// A [`Speech`](trait.Speech.html) with any built-in
/// backend.
pub enum AnySpeech {
    Espeak(EspeakSpeech),
    Say(SaySpeech)
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
            AnyVoice::Espeak(voice) => Ok(AnySpeech::Espeak(voice.speak(sentence)?)),
            AnyVoice::Say(voice)    => Ok(AnySpeech::Say(voice.speak(sentence)?)),
        }
    }
}

impl crate::Speech for AnySpeech {
    type Error = Error;

    fn await_done(&self) -> Result<(), Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.await_done()?),
            AnySpeech::Say(say)       => Ok(say.await_done()?),
        }
    }

    fn is_done(&self) -> Result<bool, Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.is_done()?),
            AnySpeech::Say(say)       => Ok(say.is_done()?),
        }
    }

    fn cancel(&self) -> Result<(), Error> {
        match self {
            AnySpeech::Espeak(espeak) => Ok(espeak.cancel()?),
            AnySpeech::Say(say)       => Ok(say.cancel()?),
        }
    }
}
