#![feature(buf_read_has_data_left)]

mod wave;
mod effects;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut file = File::open("testfiles/file_example_WAV_1MG.wav")?;
    //let mut file = File::open("testfiles/notwav.wav")?;
    let wav = wave::Wave::from_wav_file(file);
    println!("{:?}", wav);
    Ok(())
}
