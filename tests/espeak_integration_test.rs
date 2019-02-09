use tavla::{espeak, Speech, Voice};

/// If espeak is unavailable, it should not be obtainable (Err).
/// If available, it must be callable.
#[test]
fn speak_espeak() {
    match espeak() {
        Err(err) => {
            // not being available is an ok outcome, test successful
            println!("espeak not available: {:?}.", err);
        }
        // If it is, it must be invokable successfully
        Ok(espeak) => {
            espeak
                .speak("Hello with espeak.... And hello again after a long _pause_.")
                .expect("espeak obtained, but failed to speak a phrase")
                .await_done()
                .expect("espeak obtained, but failed to speak a phrase until done");

            println!("espeak available: {:?}", espeak);
        }
    }
}
