use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::fs;

pub async fn record_and_transcribe() -> Result<String, String> {
    let temp_wav = NamedTempFile::new().map_err(|e| format!("Ошибка создания temp-файла: {}", e))?;
    let wav_path = temp_wav.path().to_path_buf();

    record_audio(&wav_path)?;

    let whisper_output = transcribe_with_whisper(&wav_path).await?;
    let _ = fs::remove_file(wav_path).await;
    Ok(whisper_output.trim().to_string())
}

fn record_audio(output_path: &std::path::Path) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("Нет устройства ввода")?;
    let config = device.default_input_config().map_err(|e| format!("Ошибка конфигурации: {}", e))?;

    let spec = WavSpec {
        channels: config.channels() as u16,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = Arc::new(Mutex::new(
        WavWriter::create(output_path, spec).map_err(|e| format!("Ошибка создания WAV: {}", e))?
    ));

    let writer_clone = writer.clone();
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut w = writer_clone.lock().unwrap();
            for &sample in data {
                let amp = (sample * i16::MAX as f32) as i16;
                let _ = w.write_sample(amp);
            }
        },
        move |err| eprintln!("Ошибка записи: {}", err),
        None,
    ).map_err(|e| format!("Ошибка создания потока: {}", e))?;

    stream.play().map_err(|e| format!("Ошибка запуска записи: {}", e))?;
    println!("Запись... Говорите (5 секунд)");
    thread::sleep(Duration::from_secs(5));
    drop(stream);
    writer.lock().unwrap().finalize().map_err(|e| format!("Ошибка сохранения WAV: {}", e))?;
    Ok(())
}

async fn transcribe_with_whisper(wav_path: &std::path::Path) -> Result<String, String> {
    let output = Command::new("whisper-cli")
        .arg("-m")
        .arg("/usr/share/whisper/models/ggml-base.bin")
        .arg("-f")
        .arg(wav_path)
        .output()
        .await
        .map_err(|e| format!("whisper-cli не найден: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ошибка распознавания: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let transcript = lines.last().unwrap_or(&"").trim();
    if transcript.is_empty() {
        Err("Не удалось распознать речь".to_string())
    } else {
        Ok(transcript.to_string())
    }
}
