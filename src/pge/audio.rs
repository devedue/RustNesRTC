use alto::sys::ALint;
use alto::sys::{ALCcontext, ALCdevice, ALuint, AlApi, ALshort};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use libc::c_void;

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

    audio_thread_handle: Option<JoinHandle<()>>,
    audio_thread_active: AtomicBool,
    global_time: AtomicBool,
    func_user_synth: fn(u32, f32, f32) -> f32,
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

            audio_thread_handle: None,
            audio_thread_active: AtomicBool::from(false),
            global_time: AtomicBool::from(false),
            func_user_synth: |_, _, _| -> f32 {
                return 0.0;
            },
        };
    }

    pub fn set_user_synth_function(&mut self, func: fn(u32, f32, f32) -> f32) {
        self.func_user_synth = func;
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
            self.audio_thread_active = AtomicBool::from(false);
            self.sample_rate = sample_rate;
            self.channels = channels;
            self.block_count = blocks;
            self.block_samples = block_samples;
            self.block_memory = Vec::new();

            // Open the device and create the context
            let new_device = self.al.alcOpenDevice(std::ptr::null());
            if new_device != std::ptr::null_mut() {
                self.device = Arc::new(new_device);
                self.context = Arc::new(self.al.alcCreateContext(*self.device.as_ref(), std::ptr::null()));
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
            self.al.alGenSources(1, &mut new_source);
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

            self.audio_thread_active.store(true, Ordering::Relaxed);
            // self.audio_thread_handle = Some(std::thread::spawn(move || {
            //     self.audio_thread();
            // }));
            return true;
        }
    }

    // Stop and clean up audio system
    fn destroy_audio(&mut self) -> bool {
        unsafe {
            self.audio_thread_active.store(false, Ordering::Relaxed);

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
            const TIME_STEP: f32 = 1.0 / 44100.0;

            let f_max_sample = 32766.0;

            let mut v_processed = Vec::<ALuint>::new();

            while *self.audio_thread_active.get_mut() {
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

                    let func = self.func_user_synth;
                    new_sample = (func(0, global_time, TIME_STEP).clamp(-1.0,1.0) * f_max_sample) as i16;
                    self.block_memory[n as usize] = new_sample;

                    global_time = global_time + TIME_STEP;
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
                    .alSourceQueueBuffers(self.source.unwrap(), 1, &last);
                // Remove it from ours
                

                // If it's not playing for some reason, change that
                if state != alto::sys::AL_PLAYING {
                    self.al.alSourcePlay(self.source.unwrap());
                }
            }
        }
    }
}
