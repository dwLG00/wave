use crate::wave;

pub fn compressor(audio: Vec<f64>, cutoff: f64, ratio: f64) -> Vec<f64> {
    // Compressor effect

    let mut buffer = Vec::<f64>::new();
    for sample in audio {
        if sample.abs() < cutoff {
            buffer.push(sample);
        } else {
            let polarity: f64 = if sample > 0.0 { 1.0 } else { -1.0 };
            let amount_above = sample.abs() - cutoff;
            let compressed_amount = polarity * (amount_above / ratio + cutoff);
            buffer.push(compressed_amount);
        }
    }
    buffer
}
