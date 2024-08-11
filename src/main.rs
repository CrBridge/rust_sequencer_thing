use std::vec;
use rodio::{OutputStream, Sink};
use rodio::source::Source;
use rodio::buffer::SamplesBuffer;
use rand::Rng;

fn main(){

    // This decides the composition, vector index indicates the loop position
    // For a longer loop (e.g., 8 loops instead of 4) extend the length of each vector
    // (make sure each vector is of equal length though)
    let kick_order: Vec<u8> =   [1, 1, 1, 1].to_vec();
    let snare_order: Vec<u8> =  [1, 0, 1, 0].to_vec();
    let hat_order: Vec<u8> =    [1, 1, 1, 1].to_vec();
    let bpm = 120.0;

    // Creating the sounds
    // kick drum
    let mut kick = generate_pitch_envelope_wave(110.0, 20.0, 0.5, 44100);
    apply_envelope(&mut kick, 44100, 0.01, 0.3);
    low_pass_filter(&mut kick, 44100, 11000.0);

    // snare (not really, but im calling it that)
    let mut snare = generate_noise(0.9, 44100);
    apply_envelope(&mut snare, 44100, 0.01, 0.3);
    high_pass_filter(&mut snare, 44100, 1000.0);
    low_pass_filter(&mut snare, 44100, 2000.0);

    // hi hat
    let mut hat = generate_noise(0.2, 44100);
    apply_envelope(&mut hat, 44100, 0.01, 0.2);
    high_pass_filter(&mut hat, 44100, 300.0);

    // rodio playback setup
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // little convoluted, but we create the loop for each instrument, then convert that loop
    // into a Sample Buffer, so that we can mix them together using rodio
    let kick_beat = generate_loop(kick_order, &mut kick, bpm);
    let kick_buffer = SamplesBuffer::new(1, 44100, kick_beat);
    let snare_beat = generate_loop(snare_order, &mut snare, bpm);
    let snare_buffer = SamplesBuffer::new(1, 44100, snare_beat);
    let hat_beat = generate_loop(hat_order, &mut hat, bpm);
    let hat_buffer = SamplesBuffer::new(1, 44100, hat_beat);
    // combine all 3 buffers together, then set it to repeat infinitely
    let beat_source = kick_buffer.mix(snare_buffer).mix(hat_buffer).repeat_infinite();
    // add the source object into a sink, which is set to begin playback immediately
    sink.append(beat_source);

    println!("Playing Sequence...\n use Ctrl+C to exit because I was too lazy to program that");
    // Audio playback is done in a background thread, so the program will end straight away
    // unless we call this to sleep the main thread until the audio in the sink is finished
    // which it never will as its set to repeat
    sink.sleep_until_end();

}

// self explanatory, creates silence as each sample has a value of 0
fn generate_silence(duration: f32, sample_rate: u32) -> Vec<f32> {
    // As audio uses samples, we need to convert the duration
    // to the sample size equivalent by multiplying by the sampling rate
    let sample_count = (duration * sample_rate as f32) as usize;
    vec![0.0; sample_count]
}

// Currently just generates white noise of a given duration
// left in commented out code for some more interesting noises
fn generate_noise(duration: f32, sample_rate: u32) -> Vec<f32> {
    let sample_count = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(sample_count);

    for _i in 0..sample_count {
        // the t variable is needed for making other wave (sine, saw, etc.)
        //let t = i as f32 / sample_rate as f32;
        let sample = rand::thread_rng().gen_range(0.0..1.0) * 0.5 - 0.25; // White Noise

        // The following is some other wave/sound types I was playing around with

        // Sine Wave
        //let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        // Saw Wave
        //let sample = 2.0 * (frequency * t - (frequency * t - 0.5).floor());
        // Square Wave, vary 0.0 for pulse
        //let sample = if ((2.0 * std::f32::consts::PI * frequency * t).sin())
        //              > 0.0 {0.3} else {-0.3};// Square Wave, vary 0.0 for pulse
        // Triangle Wave
        //let sample = 2.0 * (2.0 * (frequency * t).fract() - 1.0).abs() - 1.0;

        // 2-oscillator sine wave
        // need to look into it more if I want it
        //let sample = (2.0 * std::f32::consts::PI * frequency * t).sin()
            //               + (2.0 * std::f32::consts::PI * frequency * 1.001 * t).sin();

        // 2-oscillator triangle wave
        //let sample = 2.0 * (2.0 * (frequency * t).fract() - 1.0).abs() - 1.0
          //                 + 2.0 * (2.0 * (frequency * 1.01 * t).fract() - 1.0).abs() - 1.0;

        // sine wave with some white noise on top
        //let sample = (2.0 * std::f32::consts::PI * frequency * t).sin()
            //               + rand::thread_rng().gen_range(0.0..1.0) * 0.25 - 0.125;
        samples.push(sample);
    }

    return samples;
}

// real simple attack-decay envelope
fn apply_envelope(samples: &mut [f32], sample_rate: u32, attack_time: f32, decay_time: f32) {
    let attack_samples = (attack_time * sample_rate as f32) as usize;
    let decay_samples = (decay_time * sample_rate as f32) as usize;

    for i in 0..samples.len() {
        // volume begins at/near 0
        // and linearly increases until it reaches the given attack duration
        if i < attack_samples {
            // Linear attack ramp
            samples[i] *= i as f32 / attack_samples as f32;
        // it then immediately decays as I forego sustain and release
        // (might be more apt to call this an attack-release envelope)
        } else if i < attack_samples + decay_samples {
            samples[i] *= 1.0 - ((i - attack_samples) as f32 / decay_samples as f32);
        // past the decay, remaining samples are set to silent
        } else {
            samples[i] = 0.0;
        }
    }
}

// high pass : higher frequencies stay, frequencies above cutoff are reduced
fn high_pass_filter(samples: &mut [f32], sample_rate: u32, cutoff_freq: f32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);

    let mut previous_output = 0.0;
    let mut previous_input = 0.0;

    for sample in samples.iter_mut() {
        let current_input = *sample;
        let output = alpha * (previous_output + current_input - previous_input);

        previous_output = output;
        previous_input = current_input;

        // Update the current sample with the filtered output
        *sample = output;
    }
}

// low pass : lower frequencies stay, frequencies below the cutoff are reduced
fn low_pass_filter(samples: &mut [f32], sample_rate: u32, cutoff_freq: f32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);

    let mut previous_output = 0.0;

    for sample in samples.iter_mut() {
        let output = alpha * *sample + (1.0 - alpha) * previous_output;
        previous_output = output;

        // Update the current sample with the filtered output
        *sample = output;
    }
}

// unused function to reduce the sampling resolution, was trying to recreate some
// kind of 8-bit-esque effect
// I dont know if its technically a bitcrusher by definition, just what I called it
/*fn bitcrush(samples: &mut [f32]) {
    let mut x = 0.0;
    for i in 0..samples.len() {
        if i % 5 == 0 {
            x = samples[i];
        }
        else {
            samples[i] = x;
        }
    }
}*/

// sine wave generation with a lerping frequency
fn generate_pitch_envelope_wave(start_frequency: f32, end_frequency: f32, duration: f32,
    sample_rate: u32) -> Vec<f32> {
    let sample_count = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(sample_count);

    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        
        // Lerp between start_freq and end_freq
        let current_frequency = start_frequency + (end_frequency - start_frequency) * (t / duration);
        
        // Calculate wave value for current time
        let sample = (2.0 * std::f32::consts::PI * current_frequency * t).sin();
        samples.push(sample);
    }

    return samples;
}

fn generate_loop(order: Vec<u8>, sound: &mut Vec<f32>, bpm: f32) -> Vec<f32> {
    // use bpm value, to calculate the time in seconds of a single beat
    let note_duration = 60.0 / bpm; 
    // generate silence with duration of one beat
    let silence = generate_silence(note_duration, 44100);
    // change length of sound vector to that of one beat
    // if increased, vector is padded with 0, if decrease, tail samples are cut off
    sound.resize(silence.len(), 0.0);
    
    let mut beat: Vec<f32> = Vec::new(); 
    // loop for the amount of notes being created 
    for i in 0..order.len() {
        // a value of 0 represents no sound played for that point, so add silence
        if order[i] == 0 {beat.extend(silence.iter().cloned())}
        // a value of 1 represents a note, so add the sound
        else if order[i] == 1 {beat.extend(sound.iter().cloned())}
    }
    return beat;
 }

