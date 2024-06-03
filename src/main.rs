#![feature(buf_read_has_data_left)]

mod wave;
mod effects;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut file = File::open("testfiles/file_example_WAV_1MG.wav")?;
    //let mut file = File::open("testfiles/notwav.wav")?;
    let wav = wave::Wave::from_wav_file(file);

    let compress = |x: Vec<i16>| -> Vec<i16> { effects::compressor(x, i16::MAX / 5, 10.0)};
    let delay = |x: Vec<i16>| -> Vec<i16> { effects::delay(x, 44100, 0.2) };

    //let wav2 = wave::apply(wav, compress);
    let wav2 = wave::apply(wav, delay);
    println!("{:?}", wav2);
    let mut file2 = File::create("testfiles/file_example_out.wav")?;
    wav2.to_wav_file(file2);
    Ok(())
}
