use crate::wave;

pub fn compressor(audio: Vec<i16>, cutoff: i16, ratio: f64) -> Vec<i16> {
    // Compressor effect

    let mut buffer = Vec::<i16>::new();
    for sample in audio {
        if sample.abs() < cutoff {
            buffer.push(sample);
        } else {
            let polarity: i16 = if sample > 0 { 1 } else { -1 };
            let amount_above = sample.abs() - cutoff;
            let compressed_amount = polarity * ( ((amount_above as f64) / ratio) as i16 + cutoff);
            buffer.push(compressed_amount);
        }
    }
    buffer
}

pub fn delay(audio: Vec<i16>, delay_by: usize, amt: f64) -> Vec<i16> {
    // Delay effect
    let mut buffer = Vec::<i16>::new();
    for i in 0..delay_by {
        buffer.push(audio[i]);
    }
    for i in delay_by..audio.len() {
        let delayed = ((audio[i - delay_by] as f64) * amt) as i16;
        buffer.push(audio[i] + delayed);
    }
    buffer
}
