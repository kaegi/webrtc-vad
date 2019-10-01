#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
mod bindgen;
use bindgen::*;
use std::convert::TryFrom;

pub enum VadMode {
    Quality = 0,
    LowBitrate = 1,
    Aggressive = 2,
    VeryAggressive = 3,
}

#[derive(Debug)]
pub enum SampleRate {
    Rate8kHz = 8000,
    Rate16kHz = 16000,
    Rate32kHz = 32000,
    Rate48kHz = 48000,
}

impl TryFrom<i32> for SampleRate {
    type Error = &'static str;
    fn try_from(item: i32) -> Result<Self, Self::Error> {
        match item {
            8000 => Ok(SampleRate::Rate8kHz),
            16000 => Ok(SampleRate::Rate16kHz),
            32000 => Ok(SampleRate::Rate32kHz),
            48000 => Ok(SampleRate::Rate48kHz),
            _ => Err("Invalid sample rate"),
        }
    }
}

pub struct Vad {
    fvad: *mut Fvad,
}

impl Vad {
    /// Creates and initializes a VAD instance.
    ///
    /// On success, returns a pointer to the new VAD instance, which should
    /// eventually be deleted using fvad_free().
    ///
    /// Panics in case of a memory allocation error.
    ///
    /// Defaults to 8KHz sample rate and `Quality` mode.
    pub fn new() -> Self {
        Self::new_with_rate_and_mode(SampleRate::Rate8kHz, VadMode::Quality)
    }

    /// Creates and initializes a VAD instance.
    ///
    /// On success, returns a pointer to the new VAD instance, which should
    /// eventually be deleted using fvad_free().
    ///
    /// Panics in case of a memory allocation error.
    ///
    /// Defaults to `Quality` mode.
    ///
    /// Valid values for the sample rate are 8000, 16000, 32000 and 48000. The default is 8000. Note
    /// that internally all processing will be done 8000 Hz; input data in higher
    /// sample rates will just be downsampled first.
    pub fn new_with_rate(sample_rate: SampleRate) -> Self {
        Self::new_with_rate_and_mode(sample_rate, VadMode::Quality)
    }

    /// Creates and initializes a VAD instance.
    ///
    /// On success, returns a pointer to the new VAD instance, which should
    /// eventually be deleted using fvad_free().
    ///
    /// Panics in case of a memory allocation error.
    ///
    /// Defaults to `8000` sample rate.
    pub fn new_with_mode(mode: VadMode) -> Self {
        Self::new_with_rate_and_mode(SampleRate::Rate8kHz, mode)
    }

    /// Creates and initializes a VAD instance.
    ///
    /// On success, returns a pointer to the new VAD instance, which should
    /// eventually be deleted using fvad_free().
    ///
    /// Panics in case of a memory allocation error.
    ///
    /// Valid values for the sample rate are 8000, 16000, 32000 and 48000. The default is 8000. Note
    /// that internally all processing will be done 8000 Hz; input data in higher
    /// sample rates will just be downsampled first.
    pub fn new_with_rate_and_mode(sample_rate: SampleRate, mode: VadMode) -> Self {
        unsafe {
            let fvad = fvad_new();
            if fvad.is_null() {
                panic!("fvad_new() did not return a valid instance (memory allocation error)");
            }
            let mut instance = Vad { fvad };
            instance.set_sample_rate(sample_rate);
            instance.set_mode(mode);
            instance
        }
    }

    /// Reinitializes a VAD instance, clearing all state and resetting mode and
    /// sample rate to defaults.
    pub fn reset(&mut self) {
        unsafe {
            fvad_reset(self.fvad);
        }
    }

    /// Sets the input sample rate in Hz for a VAD instance.
    ///
    /// Valid values are 8000, 16000, 32000 and 48000. The default is 8000. Note
    /// that internally all processing will be done 8000 Hz; input data in higher
    /// sample rates will just be downsampled first.
    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
        let sample_rate = sample_rate as i32;
        unsafe {
            assert_eq!(fvad_set_sample_rate(self.fvad, sample_rate), 0);
        }
    }

    /// Changes the VAD operating ("aggressiveness") mode of a VAD instance.
    ///
    /// A more aggressive (higher mode) VAD is more restrictive in reporting speech.
    /// Put in other words the probability of being speech when the VAD returns 1 is
    /// increased with increasing mode. As a consequence also the missed detection
    /// rate goes up.
    ///
    /// Valid modes are 0 ("quality"), 1 ("low bitrate"), 2 ("aggressive"), and 3
    /// ("very aggressive"). The default mode is 0.
    pub fn set_mode(&mut self, mode: VadMode) {
        let mode = mode as i32;

        unsafe { assert_eq!(fvad_set_mode(self.fvad, mode), 0) }
    }

    /// Calculates a VAD decision for an audio frame.
    ///
    /// `frame` is an array of `length` signed 16-bit samples. Only frames with a
    /// length of 10, 20 or 30 ms are supported, so for example at 8 kHz, `length`
    /// must be either 80, 160 or 240.
    ///
    /// Returns              : Ok(true) - (active voice),
    ///                       Ok(false) - (non-active Voice),
    ///                       Err(()) - (invalid frame length).
    pub fn is_voice_segment(&mut self, buffer: &[i16]) -> Result<bool, ()> {
        let b = &buffer[0] as *const i16;

        unsafe {
            match fvad_process(self.fvad, b, buffer.len()) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(()),
            }
        }
    }
}

impl Drop for Vad {
    fn drop(&mut self) {
        unsafe {
            fvad_free(self.fvad);
        }
    }
}

impl Default for Vad {
    fn default() -> Self {
        Vad::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_sample_rate() {
        let mut vad = Vad::new();
        assert_eq!(
            vad.set_sample_rate(SampleRate::try_from(8000i32).unwrap()),
            ()
        );
        assert_eq!(vad.set_sample_rate(SampleRate::Rate8kHz), ());
    }

    #[test]
    fn is_voice_segment() {
        let mut vad = Vad::new();

        let buffer = std::iter::repeat(0).take(160).collect::<Vec<i16>>();
        assert_eq!(vad.is_voice_segment(buffer.as_slice()), Ok(false));
    }

    #[test]
    fn set_mode() {
        let mut vad = Vad::new();
        assert_eq!(vad.set_mode(VadMode::Quality), ());
    }
}
