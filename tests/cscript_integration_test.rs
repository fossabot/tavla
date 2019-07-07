extern crate tavla;
#[cfg(target_os = "windows")]
use tavla::{cscript_voice, Speech, Voice};

#[cfg(target_os = "windows")]
#[test]
fn speak_cscript() {
    match cscript_voice() {
        Err(err) => {
            // not being available is an ok outcome, test successful
            println!("cscript not available: {:?}.", err);
        }
        // If it is, it must be invokable successfully
        Ok(cscript) => {
            cscript
                .speak("Hello with C script.... And hello again after a long _pause_.")
                .expect("cscript obtained, but failed to speak a phrase")
                .await_done()
                .expect("cscript obtained, but failed to speak a phrase until done");

            println!("cscript available: {:?}", cscript);
        }
    }
}
