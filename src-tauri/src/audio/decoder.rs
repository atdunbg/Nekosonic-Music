//! 音频解码
//!
//! 基于 symphonia 解码音频，并提供采样格式转换、声道重混、重采样等工具函数。
//! `decode_to_ring` 是核心入口，将解码后的样本写入环形缓冲区供播放回调消费。

use ringbuf::traits::Producer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

use crate::audio::buffer::RingProducer;

/// 将 Symphonia 解码后的音频缓冲区转换为交错排列的 f32 采样数据
fn convert_to_interleaved_f32(decoded: &AudioBufferRef) -> Vec<f32> {
    let channels = decoded.spec().channels.count();
    let frames = decoded.frames();
    let mut out = Vec::with_capacity(frames * channels);

    match decoded {
        AudioBufferRef::U8(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u8::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u16::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::U24(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame].0 as f32 / 8388607.0);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / u32::MAX as f32 * 2.0 - 1.0);
                }
            }
        }
        AudioBufferRef::S8(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i8::MAX as f32);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i16::MAX as f32);
                }
            }
        }
        AudioBufferRef::S24(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame].0 as f32 / 8388607.0);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32 / i32::MAX as f32);
                }
            }
        }
        AudioBufferRef::F32(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame]);
                }
            }
        }
        AudioBufferRef::F64(buf) => {
            for frame in 0..frames {
                for ch in 0..channels {
                    out.push(buf.chan(ch)[frame] as f32);
                }
            }
        }
    }

    out
}

/// 重混声道数，将交错采样数据从源声道数转换为目标声道数
fn remix_channels(
    interleaved: &[f32],
    src_channels: u16,
    target_channels: u16,
    src_frames: usize,
) -> Vec<f32> {
    if src_channels == target_channels {
        return interleaved.to_vec();
    }

    let src_ch = src_channels as usize;
    let tgt_ch = target_channels as usize;
    let mut out = Vec::with_capacity(src_frames * tgt_ch);

    if src_ch == 1 && tgt_ch == 2 {
        for &s in interleaved {
            out.push(s);
            out.push(s);
        }
    } else if src_ch == 2 && tgt_ch == 1 {
        for i in 0..src_frames {
            let l = interleaved[i * 2];
            let r = interleaved[i * 2 + 1];
            out.push((l + r) * 0.5);
        }
    } else {
        for i in 0..src_frames {
            for ch in 0..tgt_ch {
                let src_ch_idx = ch.min(src_ch.saturating_sub(1));
                out.push(interleaved[i * src_ch + src_ch_idx]);
            }
        }
    }

    out
}

/// 对解码音频进行重采样和声道重混，输出目标采样率和声道数的交错 f32 数据
fn resample_and_remix(
    decoded: &AudioBufferRef,
    target_sample_rate: u32,
    target_channels: u16,
    src_rate: f64,
    src_channels: u16,
) -> Vec<f32> {
    let interleaved = convert_to_interleaved_f32(decoded);
    let src_frames = if src_channels > 0 {
        interleaved.len() / src_channels as usize
    } else {
        0
    };

    if src_frames == 0 {
        return Vec::new();
    }

    let remixed = remix_channels(&interleaved, src_channels, target_channels, src_frames);
    let remixed_ch = target_channels as usize;

    let ratio = target_sample_rate as f64 / src_rate;
    let need_resample = (ratio - 1.0).abs() > 0.001;

    if !need_resample {
        return remixed;
    }

    let target_frames = (src_frames as f64 * ratio).round() as usize;
    if target_frames == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(target_frames * remixed_ch);
    for i in 0..target_frames {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;
        let next_idx = (src_idx + 1).min(src_frames - 1);

        for ch in 0..remixed_ch {
            let s0 = remixed[src_idx * remixed_ch + ch];
            let s1 = remixed[next_idx * remixed_ch + ch];
            out.push(s0 + (s1 - s0) * frac as f32);
        }
    }

    out
}

/// 将音频数据解码并写入环形缓冲区，供播放回调消费
pub fn decode_to_ring(
    mss: MediaSourceStream,
    mut producer: RingProducer,
    playing: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    decode_done: Arc<AtomicBool>,
    seek_time: Option<f64>,
    target_sample_rate: u32,
    target_channels: u16,
) {
    let hint = Hint::new();
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[audio] 探测格式失败: {}", e);
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    let mut format_reader = probed.format;
    let track = match format_reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    {
        Some(t) => t,
        None => {
            eprintln!("[audio] 未找到有效音频轨道");
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    let track_id = track.id;
    let codec_params = &track.codec_params;
    let src_rate = codec_params.sample_rate.unwrap_or(44100) as f64;
    let src_channels = codec_params.channels.unwrap_or_else(|| {
        symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT
    }).count() as u16;

    let mut decoder = match symphonia::default::get_codecs().make(codec_params, &decoder_opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[audio] 创建解码器失败: {}", e);
            decode_done.store(true, Ordering::Relaxed);
            return;
        }
    };

    if let Some(time) = seek_time {
        let seek_to = SeekTo::Time {
            time: Time::from(time),
            track_id: Some(track_id),
        };
        if let Err(e) = format_reader.seek(SeekMode::Accurate, seek_to) {
            eprintln!("[audio] seek 失败(time={}s): {}", time, e);
        }
    }

    let ratio = target_sample_rate as f64 / src_rate;
    let need_resample = (ratio - 1.0).abs() > 0.001;
    let need_remix = src_channels != target_channels;

    while !cancelled.load(Ordering::Relaxed) {
        let packet = match format_reader.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => continue,
            Err(e) => {
                eprintln!("[audio] 读取包失败: {}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[audio] 解码失败: {}", e);
                continue;
            }
        };

        let samples = if need_resample || need_remix {
            resample_and_remix(&decoded, target_sample_rate, target_channels, src_rate, src_channels)
        } else {
            convert_to_interleaved_f32(&decoded)
        };

        let mut write_pos = 0;
        while write_pos < samples.len() && !cancelled.load(Ordering::Relaxed) {
            let remaining = &samples[write_pos..];
            let n = producer.push_slice(remaining);
            if n == 0 {
                if !playing.load(Ordering::Relaxed) {
                    while !playing.load(Ordering::Relaxed) && !cancelled.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            write_pos += n;
        }
    }

    decode_done.store(true, Ordering::Relaxed);
}
