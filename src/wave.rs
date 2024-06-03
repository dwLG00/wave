use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::fmt;

#[derive(Clone)]
pub struct Wave {
    channel_left: Vec<i16>,
    channel_right: Vec<i16>,
    sample_rate: u32
}

impl fmt::Debug for Wave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wave")
            .field("channel_left.len", &self.channel_left.len())
            .field("channel_right.len", &self.channel_right.len())
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

impl Wave {
    pub fn from_wav_file(file: File) -> Wave {
        // Opens a wav file and turns it into a Wave struct

        // Buffers into which we read the contents into (and then convert)
        let mut u16_buffer: [u8; 2] = [0; 2];
        let mut u32_buffer: [u8; 4] = [0; 4];
        let mut i16_buffer: [u8; 2] = [0; 2];

        // The reader itself
        let mut buf_reader = BufReader::new(file);

        // Bytes 1-4: 'RIFF'
        buf_reader.read(&mut u32_buffer);
        assert!(b2u32(u32_buffer, Endian::Big) == 0x52494646); // 'RIFF' == 0x52494646

        // Bytes 5-8: file size - 8 bytes
        buf_reader.read(&mut u32_buffer);
        let file_size = b2u32(u32_buffer, Endian::Little) + 8; // File size in bytes

        // Bytes 9-12: 'WAVE'
        buf_reader.read(&mut u32_buffer);
        assert!(b2u32(u32_buffer, Endian::Big) == 0x57415645); // 'WAVE' == 0x57415645

        // FMT Block
        // Bytes 13-16: fmt block identifier (0x66, 0x6D, 0x74, 0x20)
        buf_reader.read(&mut u32_buffer);
        assert!(b2u32(u32_buffer, Endian::Big) == 0x666D7420);

        // Bytes 17-20: Chunk size - 8 bytes. In this case 16.
        buf_reader.read(&mut u32_buffer);
        let block_size = b2u32(u32_buffer, Endian::Little) + 8;

        // Bytes 21-22: Audio Format
        buf_reader.read(&mut u16_buffer);
        let audio_format = b2u16(u16_buffer, Endian::Little);
        assert!(audio_format == 1); // 1 = PCM, 3 = IIEE float

        // Bytes 23-24: Number of Channels
        buf_reader.read(&mut u16_buffer);
        let n_channels = b2u16(u16_buffer, Endian::Little);

        // Bytes 25-28: Sample Rate
        buf_reader.read(&mut u32_buffer);
        let sample_rate = b2u32(u32_buffer, Endian::Little);

        // Bytes 29-32: BytePerSec (frequency * BytePerBloc)
        buf_reader.read(&mut u32_buffer);
        let bytes_per_sec = b2u32(u32_buffer, Endian::Little);

        // Bytes 33-34: BytePerBloc (#Channels * BitsPerSample / 8)
        buf_reader.read(&mut u16_buffer);
        let bytes_per_bloc = b2u16(u16_buffer, Endian::Little);

        // Byte 35-36: BitsPerSample
        buf_reader.read(&mut u16_buffer);
        let bits_per_sample = b2u16(u16_buffer, Endian::Little);

        assert!(bits_per_sample / 8 * 8 == bits_per_sample);
        assert!(sample_rate * (bytes_per_bloc as u32) == bytes_per_sec);
        assert!(n_channels * bits_per_sample / 8 == bytes_per_bloc);
        assert!(bits_per_sample == 16); // We only allow 16-bit signed int PCM data for now

        let mut left_channel = Vec::<i16>::new();
        let mut right_channel = Vec::<i16>::new();

        //while buf_reader.has_data_left().expect("File IO Error") {
        // Block Data
        // Block Byte 1-4: 'data'
        buf_reader.read(&mut u32_buffer);
        let header_val = b2u32(u32_buffer, Endian::Big);
        assert!(header_val == 0x64617461, "Expected 0x64617461, got {:#01x}", header_val); // 0x64617461 == 'data'

        // Block Byte 5-8: Data Size
        buf_reader.read(&mut u32_buffer);
        let data_size = b2u32(u32_buffer, Endian::Little); // Size of the sampled data

        let seek_times = data_size / (bytes_per_bloc as u32); // Number of times we loop through the block data
        match n_channels {
            1 => {
                for i in 0..seek_times {
                    buf_reader.read(&mut i16_buffer);
                    let sampled_value = b2i16(i16_buffer, Endian::Little);
                    left_channel.push(sampled_value);
                    right_channel.push(sampled_value);
                }
            },
            2 => {
                for i in 0..seek_times {
                    buf_reader.read(&mut i16_buffer);
                    left_channel.push(b2i16(i16_buffer, Endian::Little));
                    buf_reader.read(&mut i16_buffer);
                    right_channel.push(b2i16(i16_buffer, Endian::Little));
                }
            }
            _ => {}
        }

        Wave { channel_left: left_channel, channel_right: right_channel, sample_rate: sample_rate }
    }

    pub fn to_wav_file(&self, mut file: File) {
        // Writes the contents of wave struct to a wav file

        // Assemble all relevant constants
        let audio_format: u16 = 1; // PCM
        let n_channels: u16 = 2; // Stereo
        let sample_rate = self.sample_rate;
        let bits_per_sample: u16 = 16;
        let bytes_per_bloc = n_channels * bits_per_sample / 8;
        let bytes_per_sec: u32 = (bytes_per_bloc as u32) * sample_rate;
        let chunk_size: u32 = (self.channel_left.len() as u32) * 4; // channel_left.len() samples * 2 channels * 2 bytes per sample
        let file_size: u32 = 36 + chunk_size;
        let RIFF: u32 = 0x52494646;
        let WAVE: u32 = 0x57415645;
        let FMT: u32 = 0x666D7420;
        let DATA: u32 = 0x64617461;

        // Write stuff
        file.write(&u32::to_be_bytes(RIFF));
        file.write(&u32::to_le_bytes(file_size));
        file.write(&u32::to_be_bytes(WAVE));

        file.write(&u32::to_be_bytes(FMT));
        file.write(&u32::to_le_bytes(16)); // Subchunk1 size
        file.write(&u16::to_le_bytes(audio_format));
        file.write(&u16::to_le_bytes(n_channels));
        file.write(&u32::to_le_bytes(sample_rate));
        file.write(&u32::to_le_bytes(bytes_per_sec));
        file.write(&u16::to_le_bytes(bytes_per_bloc));
        file.write(&u16::to_le_bytes(bits_per_sample));

        file.write(&u32::to_be_bytes(DATA));
        file.write(&u32::to_le_bytes(chunk_size));
        for i in 0..self.channel_left.len() {
            file.write(&i16::to_le_bytes(self.channel_left[i]));
            file.write(&i16::to_le_bytes(self.channel_right[i]));
        }
    }
}

pub fn compose(wave1: Wave, wave2: Wave) -> Wave {
    // Horizontal sum of two waves
    assert!(wave1.channel_left.len() == wave1.channel_right.len());
    assert!(wave2.channel_left.len() == wave2.channel_right.len());
    assert!(wave1.sample_rate == wave2.sample_rate);

    let mut left = wave1.channel_left.clone();
    let mut right = wave1.channel_right.clone();
    left.extend_from_slice(&wave2.channel_left);
    right.extend_from_slice(&wave2.channel_right);
    Wave { channel_left: left, channel_right: right, sample_rate:  wave1.sample_rate }
}

pub fn add(wave1: Wave, wave2: Wave) -> Wave {
    // Vertical sum of two wave
    assert!(wave1.channel_left.len() == wave1.channel_right.len());
    assert!(wave2.channel_left.len() == wave2.channel_right.len());
    assert!(wave1.channel_left.len() == wave2.channel_right.len());
    assert!(wave1.sample_rate == wave2.sample_rate);
    let length = wave1.channel_left.len();

    let mut left = wave1.channel_left.clone();
    let mut right = wave1.channel_right.clone();
    for i in 0..length {
        left[i] += wave2.channel_left[i];
        right[i] += wave2.channel_right[i];
    }
    Wave { channel_left: left, channel_right: right, sample_rate: wave1.sample_rate }
}

pub fn add_with_offset(wave1: Wave, wave2: Wave, offset: usize) -> Wave {
    // Vertical sum of two waves, with wave2 `offset` units after wave1
    assert!(wave1.channel_left.len() == wave1.channel_right.len());
    assert!(wave2.channel_left.len() == wave2.channel_right.len());
    let length = wave1.channel_left.len();

    let mut left: Vec<i16> = Vec::with_capacity(length + offset);
    let mut right: Vec<i16> = Vec::with_capacity(length + offset);

    for i in 0..offset {
        left[i] = wave1.channel_left[i];
        right[i] = wave1.channel_right[i];
    }
    for i in 0..length {
        left[offset + i] = wave1.channel_left[offset + i] + wave2.channel_left[i];
        right[offset + i] = wave1.channel_right[offset + i] + wave2.channel_right[i];
    }

    Wave { channel_left: left, channel_right: right, sample_rate: wave1.sample_rate }
}

pub fn apply(wave: Wave, f: &dyn Fn(Vec<i16>) -> Vec<i16>) -> Wave {
    // Applies a function to both channels of the wave
    let sample_rate = wave.sample_rate;
    let left = f(wave.channel_left);
    let right = f(wave.channel_right);
    Wave { channel_left: left, channel_right: right, sample_rate: sample_rate }
}

enum Endian {
    Big,
    Little
}

#[inline]
pub fn b2u16(buffer: [u8; 2], endian: Endian) -> u16 {
    match endian {
        Endian::Big => u16::from_be_bytes(buffer),
        Endian::Little => u16::from_le_bytes(buffer)
    }
}

#[inline]
pub fn b2u32(buffer: [u8; 4], endian: Endian) -> u32 {
    match endian {
        Endian::Big => u32::from_be_bytes(buffer),
        Endian::Little => u32::from_le_bytes(buffer)
    }
}

#[inline]
pub fn b2i16(buffer: [u8; 2], endian: Endian) -> i16 {
    match endian {
        Endian::Big => i16::from_be_bytes(buffer),
        Endian::Little => i16::from_le_bytes(buffer)
    }
}
