use std::env::temp_dir;
use std::fs::{remove_file, File};
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
            let mut speech = espeak
                .speak("Hello with espeak.... And hello again after a long _pause_.")
                .expect("espeak obtained, but failed to speak a phrase");

            // Check if cancellation works
            assert!(!speech.is_done().unwrap());
            speech.cancel().unwrap();
            assert!(speech.is_done().unwrap());
        }
    }
}

#[test]
fn speak_to_file_espeak() {
    match espeak() {
        Err(err) => {
            // not being available is an ok outcome, test successful
            println!("espeak not available: {:?}.", err);
        }
        // If it is, it must be able to write to files
        Ok(espeak) => {
            let mut tmp = temp_dir();
            tmp.push("test.wav");

            let resulting_file = espeak
                .speak_to_file(
                    "Hello with espeak.... And hello again after a long _pause_.",
                    &tmp,
                )
                .map_err(|e| format!("Failed speak to file: {:?}", e))
                .and_then(|s| {
                    s.await_done()
                        .map_err(|e| format!("Failed wait until file written: {:?}", e))
                })
                .and_then(|_| {
                    File::open(&tmp).map_err(|e| format!("Failed to open file spoken to: {:?}", e))
                });

            let resulting_file_size = resulting_file.and_then(|f| {
                f.metadata()
                    .map(|m| m.len())
                    .map_err(|e| format!("Failed obtain file metadata: {:?}", e))
            });

            if resulting_file_size.is_ok() {
                remove_file(&tmp).expect("Tempfile could not be deleted")
            }

            match resulting_file_size {
                Ok(size) => assert!(size > 256, "File was created but has unrealistic size"),
                Err(e) => panic!(
                    "espeak obtained, but failed to speak a phrase to file. Error: {}",
                    e
                ),
            }
        }
    }
}
