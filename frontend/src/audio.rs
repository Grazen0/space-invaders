use sdl2::audio::{AudioCallback, AudioCVT, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::AudioSubsystem;
use sdl2::rwops::RWops;

use core::Sound as GameSound;

#[derive(Debug, Clone)]
pub struct Sound {
    data: Vec<u8>,
    volume: f32,
    position: usize,
    loop_sound: bool,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for dst in out.iter_mut() {
            let pre_scale = *self.data.get(self.position).unwrap_or(&128);
            let scaled_signed_float = (pre_scale as f32 - 128.0) * self.volume;
            *dst = (scaled_signed_float + 128.0) as u8;
            self.position += 1;

            if self.loop_sound && self.position >= self.data.len() {
                self.position = 0;
            }
        }
    }
}

pub struct AudioManager {
    ufo: AudioDevice<Sound>,
    shoot: AudioDevice<Sound>,
    player_die: AudioDevice<Sound>,
    invader_die: AudioDevice<Sound>,
    bomp1: AudioDevice<Sound>,
    bomp2: AudioDevice<Sound>,
    bomp3: AudioDevice<Sound>,
    bomp4: AudioDevice<Sound>,
    ufo_explode: AudioDevice<Sound>,
}

impl AudioManager {
    pub fn new(audio_subsystem: AudioSubsystem) -> Result<Self, String> {
        Ok(Self {
            ufo: device_from_wav(include_bytes!("../assets/audio/0.wav"), &audio_subsystem, true)?,
            shoot: device_from_wav(include_bytes!("../assets/audio/1.wav"), &audio_subsystem, false)?,
            player_die: device_from_wav(include_bytes!("../assets/audio/2.wav"), &audio_subsystem, false)?,
            invader_die: device_from_wav(include_bytes!("../assets/audio/3.wav"), &audio_subsystem, false)?,
            bomp1: device_from_wav(include_bytes!("../assets/audio/4.wav"), &audio_subsystem, false)?,
            bomp2: device_from_wav(include_bytes!("../assets/audio/5.wav"), &audio_subsystem, false)?,
            bomp3: device_from_wav(include_bytes!("../assets/audio/6.wav"), &audio_subsystem, false)?,
            bomp4: device_from_wav(include_bytes!("../assets/audio/7.wav"), &audio_subsystem, false)?,
            ufo_explode: device_from_wav(include_bytes!("../assets/audio/8.wav"), &audio_subsystem, false)?,
        })
    }

    pub fn play(&mut self, sound: GameSound) {
        let device = self.match_device(sound);

        device.lock().position = 0;
        device.resume();
    }

    pub fn stop(&mut self, sound: GameSound) {
        let device = self.match_device(sound);
        device.pause();
    }

    pub fn stop_all(&mut self) {
        self.ufo.pause();
        self.shoot.pause();
        self.player_die.pause();
        self.invader_die.pause();
        self.bomp1.pause();
        self.bomp2.pause();
        self.bomp3.pause();
        self.bomp4.pause();
        self.ufo_explode.pause();
    }

    fn match_device(&mut self, sound: GameSound) -> &mut AudioDevice<Sound> {
        match sound {
            GameSound::UFO => &mut self.ufo,
            GameSound::Shoot => &mut self.shoot,
            GameSound::PlayerDie => &mut self.player_die,
            GameSound::InvaderDie => &mut self.invader_die,
            GameSound::Bomp1 => &mut self.bomp1,
            GameSound::Bomp2 => &mut self.bomp2,
            GameSound::Bomp3 => &mut self.bomp3,
            GameSound::Bomp4 => &mut self.bomp4,
            GameSound::UFOExplode => &mut self.ufo_explode,
        }
    }
}

pub fn device_from_wav(buf: &[u8], audio_subsystem: &AudioSubsystem, loop_sound: bool) -> Result<AudioDevice<Sound>, String> {
    let audio_spec = AudioSpecDesired { freq: None, channels: None, samples: None };
    let mut src = RWops::from_bytes(buf)?;

    let wav = AudioSpecWAV::load_wav_rw(&mut src)?;

    audio_subsystem
        .open_playback(None, &audio_spec, move |spec| {
            let cvt = AudioCVT::new(wav.format, wav.channels, wav.freq, spec.format, spec.channels, spec.freq).expect("could not initialize audio CVT");
            let data = cvt.convert(wav.buffer().to_vec());
            Sound { data, volume: 0.25, position: 0, loop_sound }
        })
}

