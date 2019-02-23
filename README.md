# tavla
Hey there! _tavla_ is a small rust library that provides a
minimal set of speech synthesis operations on Linux systems
with _espeak_ installed, as well as out-of-the-box on Mac
systems.

## Installation
Use your package manager to install `espeak` on the machine
where _tavla_ will run, e.g. on Arch Linux:

    pacman -S espeak

For Mac, no special setup is required.

Windows is currently unsupported, sorry.

## How to use
All you can do with _tavla_ right now is saying a string out
loud, optionally emphasising some words with underscores:

    extern crate tavla;

    fn main() {
        let voice = tavla::any_voice()
            .expect("Could not find espeak or say");

        voice.say("Oh _my_, the computer is _talking_!")
            .expect("Error occurred while speaking");
    }

## Limitations, Future Plans
_tavla_ was designed to be super easy to set up, but it will
probably not make you happy if you need any of the following:
* low latency (spawning a shell takes some time),
* phoneme output,
* language/voice selection,
* support for other systems than Mac or those with `espeak` installed,
* future-based async.

_tavla_ will probably gain more backends so it will work out
of the box on Windows, too. Right now the target system is
a Raspberry Pi for me and that seems to work fine.

It also would be cool to call into espeak directly instead
of all this piping business.

There are no current plans to make _tavla_ work with futures,
but it may happen once std futures are more stable.

## Alternatives
If you are doing serious speech synthesis consider using
something more sophisticated like
[speech-dispatcher](https://crates.io/crates/speech-dispatcher)
or linking against espeak directly with
[espeak-sys](https://crates.io/crates/espeak-sys).

