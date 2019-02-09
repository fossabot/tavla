use tavla::{any_voice, Voice};

/// On every target platform this should return a voice
/// that can successfully be invoked.
#[test]
fn speak_any() {
    any_voice()
        .expect("Expected at least one voice to be loadable!")
        .speak(".") //Speak only a pause
        .expect("Single pause could not be spoken with any voice.");
}
