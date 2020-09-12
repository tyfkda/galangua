//use std::f32::consts::PI;
use hound;
use std::io::BufWriter;
use std::fs::File;

/*fn write_sine(
    writer: &mut hound::WavWriter<BufWriter<File>>, sample_rate: u32,
    frequency: f32, volume: f32, duration: f32,
) -> hound::Result<()> {
    let len = (duration * sample_rate as f32).round() as usize;
    let amplitude = i8::MAX as f32 * volume;
    for i in 0..len {
        let t = i as f32 / (sample_rate as f32);
        let sample = (t * frequency * 2.0 * PI).sin();
        writer.write_sample((sample * amplitude) as i8)?;
    }
    Ok(())
}*/

fn write_square(
    writer: &mut hound::WavWriter<BufWriter<File>>, sample_rate: u32,
    frequency: f32, volume: f32, duration: f32,
) -> hound::Result<()> {
    let len = (duration * (sample_rate as f32)).round() as usize;
    let amplitude = i8::MAX as f32 * volume;
    for i in 0..len {
        let t = (((i * 4) as f32) * frequency / (sample_rate as f32)).floor() as i32;
        let sample = ((t & 2) - 1) as f32;
        writer.write_sample((sample * amplitude) as i8)?;
    }
    Ok(())
}

fn main() -> Result<(), hound::Error> {
    let filename = "attack_start.wav";
    let sample_rate = 44100;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 8,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec).unwrap();

    let volume = 1.0 / 8.0;
    let len = 20;
    for i in 0..len {
        let freq = (880 - 440 * i / len) as f32;
        let t = (i as f32) / (len as f32);
        let v = (1.0 - t.powf(2.5)) * volume;
        write_square(&mut writer, sample_rate, freq, v, 1.0 / 16.0)?;
    }
    Ok(())
}
