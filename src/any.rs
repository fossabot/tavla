use crate::{ChildError, Espeak, EspeakError, EspeakSpeech};
#[cfg(target_os = "windows")]
use crate::{CScriptVoice, CScriptVoiceError, CScriptVoiceSpeech};
#[cfg(target_os = "macos")]
pub use crate::{Say, SayError, SaySpeech};
use failure::Fail;
use std::path::Path;

/// A [`Voice`](trait.Voice.html) that works with any of
/// the built-in techniques (currently only espeak).
pub enum AnyVoice {
    #[cfg(target_os = "windows")]
    CScript(CScriptVoice),
    Espeak(Espeak),
    #[cfg(target_os = "macos")]
    Say(Say),
}

/// A [`Speech`](trait.Speech.html) with any built-in
/// backend.
pub enum AnySpeech {
    #[cfg(target_os = "windows")]
    CScript(CScriptVoiceSpeech),
    Espeak(EspeakSpeech),
    #[cfg(target_os = "macos")]
    Say(SaySpeech),
}

#[derive(Fail, Debug)]
pub enum AnyError {
    #[cfg(target_os = "windows")]
    #[fail(display = "CScript error: {}", _0)]
    CScript(CScriptVoiceError),
    #[fail(display = "espeak error: {}", _0)]
    Espeak(EspeakError),
    #[cfg(target_os = "macos")]
    #[fail(display = "say error: {}", _0)]
    Say(SayError),
    #[fail(display = "speech synthesizer communication error: {}", _0)]
    Child(ChildError),
}

#[cfg(target_os = "windows")]
impl From<CScriptVoiceError> for AnyError {
    fn from(error: CScriptVoiceError) -> Self {
        AnyError::CScript(error)
    }
}

impl From<EspeakError> for AnyError {
    fn from(error: EspeakError) -> Self {
        AnyError::Espeak(error)
    }
}

#[cfg(target_os = "macos")]
impl From<SayError> for AnyError {
    fn from(error: SayError) -> Self {
        AnyError::Say(error)
    }
}

impl From<ChildError> for AnyError {
    fn from(error: ChildError) -> Self {
        AnyError::Child(error)
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "macos")]
impl From<Say> for AnyVoice {
    fn from(say: Say) -> Self {
        AnyVoice::Say(say)
    }
}

impl crate::Voice for AnyVoice {
    type Error = AnyError;
    type Speech = AnySpeech;

    fn speak<S>(&self, sentence: S) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
    {
        match self {
            #[cfg(target_os = "windows")]
            AnyVoice::CScript(voice) => voice
                .speak(sentence)
                .map(AnySpeech::CScript)
                .map_err(From::from),
            AnyVoice::Espeak(voice) => voice
                .speak(sentence)
                .map(AnySpeech::Espeak)
                .map_err(From::from),
            #[cfg(target_os = "macos")]
            AnyVoice::Say(voice) => voice
                .speak(sentence)
                .map(AnySpeech::Say)
                .map_err(From::from),
        }
    }

    fn speak_to_file<S, P>(
        &self,
        sentence: S,
        wav_file_path: P,
    ) -> Result<Self::Speech, Self::Error>
    where
        S: AsRef<str>,
        P: AsRef<Path>,
    {
        match self {
            #[cfg(target_os = "windows")]
            AnyVoice::CScript(voice) => voice
                .speak_to_file(sentence, wav_file_path)
                .map(AnySpeech::CScript)
                .map_err(From::from),
            AnyVoice::Espeak(voice) => voice
                .speak_to_file(sentence, wav_file_path)
                .map(AnySpeech::Espeak)
                .map_err(From::from),
            #[cfg(target_os = "macos")]
            AnyVoice::Say(voice) => voice
                .speak_to_file(sentence, wav_file_path)
                .map(AnySpeech::Say)
                .map_err(From::from),
        }
    }
}

impl crate::Speech for AnySpeech {
    type Error = AnyError;

    fn await_done(&self) -> Result<(), Self::Error> {
        match self {
            #[cfg(target_os = "windows")]
            AnySpeech::CScript(speech) => speech.await_done().map_err(From::from),
            AnySpeech::Espeak(speech) => speech.await_done().map_err(From::from),
            #[cfg(target_os = "macos")]
            AnySpeech::Say(speech) => speech.await_done().map_err(From::from),
        }
    }

    fn is_done(&self) -> Result<bool, Self::Error> {
        match self {
            #[cfg(target_os = "windows")]
            AnySpeech::CScript(speech) => speech.is_done().map_err(From::from),
            AnySpeech::Espeak(speech) => speech.is_done().map_err(From::from),
            #[cfg(target_os = "macos")]
            AnySpeech::Say(speech) => speech.is_done().map_err(From::from),
        }
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(target_os = "windows")]
            AnySpeech::CScript(speech) => speech.cancel().map_err(From::from),
            AnySpeech::Espeak(speech) => speech.cancel().map_err(From::from),
            #[cfg(target_os = "macos")]
            AnySpeech::Say(speech) => speech.cancel().map_err(From::from),
        }
    }
}
