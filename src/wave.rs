use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
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

        // Bytes 17-20: Chunk size - 8 bytes
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
}

/*
pub fn compose(wave1: Wave, wave2: Wave) -> Wave {
    // Horizontal sum of two waves
    assert!(wave1.channel_left.len() == wave1.channel_right.len());
    assert!(wave2.channel_left.len() == wave2.channel_right.len());

    let mut left = wave1.channel_left.clone();
    let mut right = wave1.channel_right.clone();
    left.extend_from_slice(&wave2.channel_left);
    right.extend_from_slice(&wave2.channel_right);
    Wave { channel_left: left, channel_right: right }
}

pub fn add(wave1: Wave, wave2: Wave) -> Wave {
    // Vertical sum of two wave
    assert!(wave1.channel_left.len() == wave1.channel_right.len());
    assert!(wave2.channel_left.len() == wave2.channel_right.len());
    assert!(wave1.channel_left.len() == wave2.channel_right.len());
    let length = wave1.channel_left.len();

    let mut left = wave1.channel_left.clone();
    let mut right = wave1.channel_right.clone();
    for i in 0..length {
        left[i] += wave2.channel_left[i];
        right[i] += wave2.channel_right[i];
    }
    Wave { channel_left: left, channel_right: right }
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

    Wave { channel_left: left, channel_right: right }
}

pub fn apply(wave: Wave, f: &dyn Fn(Vec<i16>) -> Vec<i16>) -> Wave {
    // Applies a function to both channels of the wave
    let left = f(wave.channel_left);
    let right = f(wave.channel_right);
    Wave { channel_left: left, channel_right: right }
}
*/

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
