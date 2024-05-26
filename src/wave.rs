pub struct Wave {
    channel_left: Vec<f64>,
    channel_right: Vec<f64>
}

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

    let mut left: Vec<f64> = Vec::with_capacity(length + offset);
    let mut right: Vec<f64> = Vec::with_capacity(length + offset);

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

pub fn apply(wave: Wave, f: &dyn Fn(Vec<f64>) -> Vec<f64>) -> Wave {
    // Applies a function to both channels of the wave
    let left = f(wave.channel_left);
    let right = f(wave.channel_right);
    Wave { channel_left: left, channel_right: right }
}
