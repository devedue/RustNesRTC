use crate::nes::sound_out;
use alto::sys::ALint;
use alto::sys::{ALCcontext, ALCdevice, ALshort, ALuint, AlApi};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// use std::thread::JoinHandle;

use libc::c_void;

pub static AUDIO_THREAD_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct Audio {
    // For OpenAL
    al: Arc<AlApi>,
    available_buffers: Vec<ALuint>,
    buffers: Vec<ALuint>,
    source: Option<ALuint>,
    device: Arc<*mut ALCdevice>,
    context: Arc<*mut ALCcontext>,
    sample_rate: u32,
    channels: u32,
    block_count: u32,
    block_samples: u32,
    block_memory: Vec<i16>,
}

impl Audio {
    pub fn new() -> Self {
        return Audio {
            available_buffers: Vec::new(),
            buffers: Vec::new(),
            source: None,
            device: Arc::new(std::ptr::null_mut()),
            context: Arc::new(std::ptr::null_mut()),
            sample_rate: 44100,
            channels: 0,
            block_count: 0,
            block_samples: 0,
            block_memory: Vec::new(),
            al: Arc::new(AlApi::load_default().unwrap()),
        };
    }

    pub fn initialise_audio(
        &mut self,
        sample_rate: u32,
        channels: u32,
        blocks: u32,
        block_samples: u32,
    ) -> bool {
        unsafe {
            // Initialise Sound Engine
            // self.audio_thread_active = AtomicBool::from(false);
            self.sample_rate = sample_rate;
            self.channels = channels;
            self.block_count = blocks;
            self.block_samples = block_samples;
            self.block_memory = Vec::new();

            // Open the device and create the context
            let new_device = self.al.alcOpenDevice(std::ptr::null());
            if new_device != std::ptr::null_mut() {
                self.device = Arc::new(new_device);
                self.context = Arc::new(
                    self.al
                        .alcCreateContext(*self.device.as_ref(), std::ptr::null()),
                );
                self.al.alcMakeContextCurrent(*self.context.as_ref());
            } else {
                return self.destroy_audio();
            }

            // Allocate memory for sound data
            self.al.alGetError();
            self.buffers
                .resize(self.block_count as usize, ALuint::from(0 as u32));
            self.al
                .alGenBuffers(self.block_count as i32, self.buffers.as_mut_ptr());

            let mut new_source = 0;
            self.al.alGenSources(channels as i32, &mut new_source);
            self.source = Some(new_source);

            for i in 0..self.block_count {
                self.available_buffers.push(self.buffers[i as usize]);
            }

            // listActiveSamples.clear();

            // Allocate Wave|Block Memory
            self.block_memory.resize(self.block_samples as usize, 0);
            if self.block_memory.len() == 0 {
                return self.destroy_audio();
            }

            AUDIO_THREAD_ACTIVE.store(true, Ordering::Relaxed);
            // self.audio_thread_handle = Some(std::thread::spawn(move || {
            //     self.audio_thread();
            // }));
            return true;
        }
    }

    // Stop and clean up audio system
    pub fn destroy_audio(&mut self) -> bool {
        unsafe {
            println!("Destroying");
            AUDIO_THREAD_ACTIVE.store(false, Ordering::Relaxed);

            // self.audio_thread_handle.unwrap();

            self.al
                .alDeleteBuffers(self.block_count as i32, self.buffers.as_mut_ptr());

            self.al.alDeleteSources(1, &self.source.unwrap());

            self.al.alcMakeContextCurrent(std::ptr::null_mut());
            self.al.alcDestroyContext(*self.context.as_ref());
            self.al.alcCloseDevice(*self.device.as_ref());
            return false;
        }
    }

    pub fn audio_thread(&mut self) {
        unsafe {
            let mut global_time = 0.0;
            let time_step: f32 = 1.0 / (self.sample_rate as f32);

            let f_max_sample = 32766.0;

            let mut v_processed = Vec::<ALuint>::new();

            while AUDIO_THREAD_ACTIVE.load(Ordering::Relaxed) {
                let mut state: ALint = 0;
                let mut processed: ALint = 0;
                self.al
                    .alGetSourcei(self.source.unwrap(), alto::sys::AL_SOURCE_STATE, &mut state);
                self.al.alGetSourcei(
                    self.source.unwrap(),
                    alto::sys::AL_BUFFERS_PROCESSED,
                    &mut processed,
                );

                // Add processed buffers to our queue
                v_processed.resize(processed as usize, 0);
                self.al.alSourceUnqueueBuffers(
                    self.source.unwrap(),
                    processed,
                    v_processed.as_mut_ptr(),
                );

                for i in 0..v_processed.len() {
                    self.available_buffers.push(v_processed[i as usize]);
                }

                // Wait until there is a free buffer (ewww)
                if self.available_buffers.len() == 0 {
                    continue;
                }

                let mut new_sample: ALshort;

                for n in 0..self.block_samples {
                    // User Process

                    let func_val = sound_out(0, global_time, time_step);
                    new_sample = (func_val.clamp(-1.0, 1.0) * f_max_sample) as i16;
                    self.block_memory[n as usize] = new_sample;

                    global_time = global_time + time_step;
                }

                let last = self.available_buffers.pop().unwrap();

                // Fill OpenAL data buffer
                self.al.alBufferData(
                    last,
                    alto::sys::AL_FORMAT_MONO16,
                    self.block_memory.as_ptr() as *const c_void,
                    (2 * self.block_samples) as i32,
                    44100,
                );
                // Add it to the OpenAL queue
                self.al
                    .alSourceQueueBuffers(self.source.unwrap(), self.channels as i32, &last);
                // Remove it from ours

                // If it's not playing for some reason, change that
                if state != alto::sys::AL_PLAYING {
                    self.al.alSourcePlay(self.source.unwrap());
                }
            }
        }
    }
}
