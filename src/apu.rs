const PI: f64 = 3.141529;

#[derive(Default)]
struct Sequencer {
    sequence: u32,
    new_sequence: u32,
    timer: u16,
    reload: u16,
    output: u8,
}

impl Sequencer {
    fn clock(&mut self, enable: bool, func: fn(s: &mut u32) -> ()) -> u8 {
        if enable {
            self.timer = self.timer.wrapping_sub(1);
            if self.timer == 0xFFFF {
                self.timer = self.reload;
                func(&mut self.sequence);
                self.output = (self.sequence & 0x00000001) as u8;
            }
        }
        return self.output;
    }
}

#[derive(Default)]
struct LengthCounter {
    counter: u8,
}

impl LengthCounter {
    fn clock(&mut self, enable: bool, halt: bool) -> u8 {
        if !enable {
            self.counter = 0;
        } else if self.counter > 0 && !halt {
            self.counter = self.counter.wrapping_sub(1);
        }
        return self.counter;
    }
}

#[derive(Default)]
struct Envelope {
    start: bool,
    disable: bool,
    divider_count: u16,
    volume: u16,
    output: u16,
    decay_count: u16,
}

impl Envelope {
    fn clock(&mut self, b_loop: bool) {
        if !self.start {
            if self.divider_count == 0 {
                self.divider_count = self.volume;

                if self.decay_count == 0 {
                    if b_loop {
                        self.decay_count = 15;
                    }
                } else {
                    self.decay_count = self.decay_count - 1;
                }
            } else {
                self.divider_count = self.divider_count - 1;
            }
        } else {
            self.start = false;
            self.decay_count = 15;
            self.divider_count = self.volume;
        }

        if self.disable {
            self.output = self.volume;
        } else {
            self.output = self.decay_count;
        }
    }
}

struct OscPulse {
    frequency: f64,
    dutycycle: f64,
    amplitude: f64,
    harmonics: u8,
}

impl Default for OscPulse {
    fn default() -> Self {
        OscPulse {
            frequency: 0.0,
            dutycycle: 0.0,
            amplitude: 1.0,
            harmonics: 7,
        }
    }
}

impl OscPulse {
    fn sample(&self, t: f64) -> f64 {
        let mut a = 0.0;
        let mut b = 0.0;
        let p = self.dutycycle * 2.0 * PI;

        let approxnegsin = |t: f64| -> f64 {
            let mut j = t * 0.15915;
            j = j - j.floor();
            return -(20.785 * j * (j - 0.5) * (j - 1.0));
        };

        for n in 1..self.harmonics {
            let c = (n as f64) * self.frequency * 2.0 * PI * t;
            a += approxnegsin(c) / (n as f64);
            b += approxnegsin(c - p * (n as f64)) / (n as f64);
        }

        return (2.0 * self.amplitude / PI) * ((a - b) as f64);
    }
}

#[derive(Default)]
struct Sweeper {
    enabled: bool,
    down: bool,
    reload: bool,
    shift: u8,
    timer: u8,
    period: u8,
    change: u16,
    mute: bool,
}

impl Sweeper {
    fn track(&mut self, target: u16) {
        if self.enabled {
            self.change = target >> self.shift;
            self.mute = (target < 8) || (target > 0x7FF);
        }
    }

    fn clock(&mut self, target: &mut u16, channel: bool) -> bool {
        let mut changed = false;
        if self.timer == 0 && self.enabled && self.shift > 0 && !self.mute {
            if *target >= 8 && self.change < 0x07FF {
                if self.down {
                    *target = *target - self.change - (channel as u16);
                } else {
                    *target = *target + self.change;
                }
                changed = true;
            }
        }

        if self.enabled {
            if self.timer == 0 || self.reload {
                self.timer = self.period;
                self.reload = false;
            } else {
                self.timer = self.timer - 1;
            }

            self.mute = (*target < 8) || (*target > 0x7FF);
        }

        return changed;
    }
}

#[derive(Default)]
struct Channel {
    enable: bool,
    halt: bool,
    sample: f64,
    output: f64,
    seq: Sequencer,
    osc: OscPulse,
    env: Envelope,
    lc: LengthCounter,
    sweep: Sweeper,
}

#[derive(Default)]
pub struct Apu {
    pulse1: Channel,
    pulse2: Channel,
    noise: Channel,
    clock_counter: u128,
    frame_clock_counter: u128,
    global_time: f64,
    length_table: [u8; 32],
}

impl Apu {
    pub fn new() -> Self {
        return Apu {
            noise: Channel {
                seq: Sequencer {
                    sequence: 0xDBDB,
                    ..Default::default()
                },
                ..Default::default()
            },
            length_table: [
                10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48,
                20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
            ],
            ..Default::default()
        };
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => {
                match (data & 0xC0) >> 6 {
                    0x00 => {
                        self.pulse1.seq.new_sequence = 0b01000000;
                        self.pulse1.osc.dutycycle = 0.125;
                    }
                    0x01 => {
                        self.pulse1.seq.new_sequence = 0b01100000;
                        self.pulse1.osc.dutycycle = 0.250;
                    }
                    0x02 => {
                        self.pulse1.seq.new_sequence = 0b01111000;
                        self.pulse1.osc.dutycycle = 0.500;
                    }
                    0x03 => {
                        self.pulse1.seq.new_sequence = 0b10011111;
                        self.pulse1.osc.dutycycle = 0.750;
                    }
                    _ => {}
                }
                self.pulse1.seq.sequence = self.pulse1.seq.new_sequence;
                self.pulse1.halt = (data & 0x20) > 0;
                self.pulse1.env.volume = data as u16 & 0x0F;
                self.pulse1.env.disable = (data & 0x10) > 0;
            }
            0x4001 => {
                self.pulse1.sweep.enabled = (data & 0x80) > 0;
                self.pulse1.sweep.period = (data & 0x70) >> 4;
                self.pulse1.sweep.down = (data & 0x08) > 0;
                self.pulse1.sweep.shift = data & 0x07;
                self.pulse1.sweep.reload = true;
            }
            0x4002 => {
                self.pulse1.seq.reload = (self.pulse1.seq.reload & 0xFF00) | (data as u16);
            }
            0x4003 => {
                self.pulse1.seq.reload =
                    (((data as u16) & 0x07) << 8) as u16 | (self.pulse1.seq.reload & 0x00FF) as u16;
                self.pulse1.seq.timer = self.pulse1.seq.reload;
                self.pulse1.seq.sequence = self.pulse1.seq.new_sequence;
                self.pulse1.lc.counter = self.length_table[((data & 0xF8) >> 3) as usize];
                self.pulse1.env.start = true;
            }
            0x4004 => {
                match (data & 0xC0) >> 6 {
                    0x00 => {
                        self.pulse2.seq.new_sequence = 0b01000000;
                        self.pulse2.osc.dutycycle = 0.125;
                    }
                    0x01 => {
                        self.pulse2.seq.new_sequence = 0b01100000;
                        self.pulse2.osc.dutycycle = 0.250;
                    }
                    0x02 => {
                        self.pulse2.seq.new_sequence = 0b01111000;
                        self.pulse2.osc.dutycycle = 0.500;
                    }
                    0x03 => {
                        self.pulse2.seq.new_sequence = 0b10011111;
                        self.pulse2.osc.dutycycle = 0.750;
                    }
                    _ => {}
                }
                self.pulse2.seq.sequence = self.pulse2.seq.new_sequence;
                self.pulse2.halt = (data & 0x20) > 0;
                self.pulse2.env.volume = data as u16 & 0x0F;
                self.pulse2.env.disable = (data & 0x10) > 0;
            }
            0x4005 => {
                self.pulse2.sweep.enabled = (data & 0x80) > 0;
                self.pulse2.sweep.period = (data & 0x70) >> 4;
                self.pulse2.sweep.down = (data & 0x08) > 0;
                self.pulse2.sweep.shift = data & 0x07;
                self.pulse2.sweep.reload = true;
            }
            0x4006 => {
                self.pulse2.seq.reload = (self.pulse2.seq.reload & 0xFF00) | (data as u16);
            }
            0x4007 => {
                self.pulse2.seq.reload =
                    (((data as u16) & 0x07) << 8) as u16 | (self.pulse2.seq.reload & 0x00FF) as u16;
                self.pulse2.seq.timer = self.pulse2.seq.reload;
                self.pulse2.seq.sequence = self.pulse2.seq.new_sequence;
                self.pulse2.lc.counter = self.length_table[((data & 0xF8) >> 3) as usize];
                self.pulse2.env.start = true;
            }
            0x400C => {
                self.noise.env.volume = (data & 0x0F) as u16;
                self.noise.env.disable = (data & 0x10) > 0;
                self.noise.halt = (data & 0x20) > 0;
            }
            0x400E => match data & 0x0F {
                0x00 => {
                    self.noise.seq.reload = 0;
                }
                0x01 => {
                    self.noise.seq.reload = 4;
                }
                0x02 => {
                    self.noise.seq.reload = 8;
                }
                0x03 => {
                    self.noise.seq.reload = 16;
                }
                0x04 => {
                    self.noise.seq.reload = 32;
                }
                0x05 => {
                    self.noise.seq.reload = 64;
                }
                0x06 => {
                    self.noise.seq.reload = 96;
                }
                0x07 => {
                    self.noise.seq.reload = 128;
                }
                0x08 => {
                    self.noise.seq.reload = 160;
                }
                0x09 => {
                    self.noise.seq.reload = 202;
                }
                0x0A => {
                    self.noise.seq.reload = 254;
                }
                0x0B => {
                    self.noise.seq.reload = 380;
                }
                0x0C => {
                    self.noise.seq.reload = 508;
                }
                0x0D => {
                    self.noise.seq.reload = 1016;
                }
                0x0E => {
                    self.noise.seq.reload = 2034;
                }
                0x0F => {
                    self.noise.seq.reload = 4068;
                }
                _ => {}
            },
            0x4015 => {
                self.pulse1.enable = (data & 0x01) > 0;
                self.pulse2.enable = (data & 0x02) > 0;
                self.noise.enable = (data & 0x04) > 0;
            }
            0x400F => {
                self.pulse1.env.start = true;
                self.pulse2.env.start = true;
                self.noise.env.start = true;
                self.noise.lc.counter = self.length_table[((data & 0xF8) >> 3) as usize]
            }
            _ => {}
        }
    }
    pub fn cpu_read(&self, addr: u8) -> u8 {
        return 0;
    }

    pub fn clock(&mut self) {
        let mut quarter_frame_clock = false;
        let mut half_frame_clock = false;

        self.global_time += 0.3333333333 / 1789773.0;

        if self.clock_counter % 6 == 0 {
            self.frame_clock_counter = self.frame_clock_counter + 1;

            if self.frame_clock_counter == 3729 {
                quarter_frame_clock = true;
            }

            if self.frame_clock_counter == 7457 {
                quarter_frame_clock = true;
                half_frame_clock = true;
            }

            if self.frame_clock_counter == 11186 {
                quarter_frame_clock = true;
            }

            if self.frame_clock_counter == 14916 {
                quarter_frame_clock = true;
                half_frame_clock = true;
                self.frame_clock_counter = 0;
            }

            // Update functional units

            // Quater frame "beats" adjust the volume envelope
            if quarter_frame_clock {
                self.pulse1.env.clock(self.pulse1.halt);
            }

            if half_frame_clock {
                self.pulse1.lc.clock(self.pulse1.enable, self.pulse1.halt);
                self.pulse1.sweep.clock(&mut self.pulse1.seq.reload, false);
            }

            self.pulse1.seq.clock(self.pulse1.enable, |s| {
                *s = ((*s & (0x0001 as u32)) << 7) | ((*s & (0x00FE as u32)) >> 1);
            });

            self.pulse1.osc.frequency = 1789773.0 / (16.0 * (self.pulse1.seq.reload + 1) as f64);
            self.pulse1.osc.amplitude = (self.pulse1.env.output.wrapping_sub(1)) as f64 / 16.0;
            self.pulse1.sample = self.pulse1.osc.sample(self.global_time);

            if self.pulse1.lc.counter > 0
                && self.pulse1.seq.timer >= 8
                && !self.pulse1.sweep.mute
                && self.pulse1.env.output > 2
            {
                self.pulse1.output =
                    self.pulse1.output + (self.pulse1.sample - self.pulse1.output) * 0.5;
            } else {
                self.pulse1.output = 0.0;
            }
            if !self.pulse1.enable {
                self.pulse1.output = 0.0;
            }

            if quarter_frame_clock {
                self.pulse2.env.clock(self.pulse2.halt);
            }

            if half_frame_clock {
                self.pulse2.lc.clock(self.pulse2.enable, self.pulse2.halt);
                self.pulse2.sweep.clock(&mut self.pulse2.seq.reload, true);
            }
            // self.pulse2.seq.clock(self.pulse2.enable, |s| {
            //     *s = ((*s & (0x0001 as u32)) << 7) | ((*s & (0x00FE as u32)) >> 1);
            // });

            self.pulse2.osc.frequency = 1789773.0 / (16.0 * (self.pulse2.seq.reload + 1) as f64);
            self.pulse2.osc.amplitude = (self.pulse2.env.output.wrapping_sub(1)) as f64 / 16.0;
            self.pulse2.sample = self.pulse2.osc.sample(self.global_time);

            if self.pulse2.lc.counter > 0
                && self.pulse2.seq.timer >= 8
                && !self.pulse2.sweep.mute
                && self.pulse2.env.output > 2
            {
                self.pulse2.output =
                    self.pulse2.output + (self.pulse2.sample - self.pulse2.output) * 0.5;
            } else {
                self.pulse2.output = 0.0;
            }

            if !self.pulse2.enable {
                self.pulse2.output = 0.0;
            }

            // Noise

            if quarter_frame_clock {
                self.noise.env.clock(self.noise.halt);
            }

            if half_frame_clock {
                // self.noise.lc.clock(self.noise.enable, self.noise.halt);
            }
            // self.noise.seq.clock(self.noise.enable, |s| {
            //     *s = (((*s & 0x0001) ^ ((*s & 0x0002) >> 1)) << 14) | ((*s & 0x7FFF) >> 1);
            // });

            self.noise.osc.frequency = 1789773.0 / (16.0 * (self.noise.seq.reload + 1) as f64);
            self.noise.osc.amplitude = (self.noise.env.output.wrapping_sub(1)) as f64 / 16.0;
            // self.noise.sample = self.noise.osc.sample(self.global_time);

            if self.noise.lc.counter > 0 && self.noise.seq.timer >= 8 {
                self.noise.output =
                    self.noise.seq.output as f64 * ((self.noise.env.output - 1) as f64 / 16.0);
            } else {
                self.noise.output = 0.0;
            }

            if !self.noise.enable {
                self.noise.output = 0.0;
            }
        }

        self.pulse1.sweep.track(self.pulse1.seq.reload);
        self.pulse2.sweep.track(self.pulse2.seq.reload);

        self.clock_counter = self.clock_counter + 1;
    }

    pub fn reset(&self) {}

    pub fn get_output_sample(&self) -> f64 {
        return ((1.0 * self.pulse1.output) - 0.8) * 0.1
            + ((1.0 * self.pulse2.output) - 0.8) * 0.1
            + (2.0 * (self.noise.output - 0.5)) * 0.1;
    }

    // fn sample_square_wave(f: f32, t: f32) -> f32 {
    //     let mut a = 0.0;
    //     let mut b = 0.0;
    //     let p = 0.5 * 2.0 * 3.14159;

    //     for n in 1..20 {
    //         let c = (n as f32) * f * 2.0 * 3.14159 * t;
    //         a = a + (c.sin() / (n as f32));
    //         b = b + ((c - p * (n as f32)).sin() / (n as f32));
    //     }
    //     return (2.0 / 3.14159) * (a - b);
    // }
}
