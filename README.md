# Rust sequencer

This was just a mix of me trying to learn Rust, sound design and some digital signal processing. Audio playback is done with the Rodio crate. Includes
some basic implementations of enveloping (pitch & attack/decay) and filtering (high & low pass). The basic sounds I made aren't the best, but it should hypothetically
be a pretty easy addition to implement using samples in lieu of those. Currently the code just loops every 4 notes length, but the loop can be extended by increasing
the capacity of the vectors storing the note orders. It also loops infinitely by default, so exiting is a little crummy. To change this and just hear the loop once
you can just remove the ```.repeat_infinite()``` snippet.
