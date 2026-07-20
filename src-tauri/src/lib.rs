use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use tauri::Emitter;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(desktop)]
use tauri::menu::{MenuBuilder, SubmenuBuilder, MenuItemBuilder};
#[cfg(desktop)]
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

/// 构建应用菜单，fullscreen_label 根据全屏状态传入 "进入全屏" 或 "退出全屏"
#[cfg(desktop)]
fn build_menu(app: &tauri::AppHandle, fullscreen_label: &str) -> tauri::Result<()> {
    let handle = app;

    let playback_menu = SubmenuBuilder::new(handle, "播放")
        .item(
            &MenuItemBuilder::with_id("play_pause", "播放 / 暂停")
                .accelerator("Space")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("next", "下一首")
                .accelerator("CmdOrCtrl+Right")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("prev", "上一首")
                .accelerator("CmdOrCtrl+Left")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("shuffle", "随机播放").build(handle)?)
        .item(&MenuItemBuilder::with_id("repeat", "循环模式").build(handle)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("vol_up", "音量增大")
                .accelerator("CmdOrCtrl+Up")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("vol_down", "音量减小")
                .accelerator("CmdOrCtrl+Down")
                .build(handle)?,
        )
        .build()?;

    let playlist_menu = SubmenuBuilder::new(handle, "播放列表")
        .item(
            &MenuItemBuilder::with_id("import", "导入音频…")
                .accelerator("CmdOrCtrl+O")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("favorite", "收藏当前曲目")
                .accelerator("CmdOrCtrl+F")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("clear", "清空播放列表").build(handle)?)
        .build()?;

    let window_menu = SubmenuBuilder::new(handle, "窗口")
        .item(
            &MenuItemBuilder::with_id("minimize", "最小化")
                .accelerator("CmdOrCtrl+M")
                .build(handle)?,
        )
        .item(&MenuItemBuilder::with_id("show", "显示窗口").build(handle)?)
        .separator()
        .item(&MenuItemBuilder::with_id("zoom", "缩放").build(handle)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("toggle_fullscreen", fullscreen_label)
                .accelerator("CmdOrCtrl+Ctrl+F")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("toggle_ontop", "窗口置顶").build(handle)?)
        .build()?;

    let help_menu = SubmenuBuilder::new(handle, "帮助")
        .item(&MenuItemBuilder::with_id("about", "关于 Hanono").build(handle)?)
        .build()?;

    let menu = MenuBuilder::new(handle)
        .item(&playback_menu)
        .item(&playlist_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    app.set_menu(menu)?;
    Ok(())
}

struct DbState {
    conn: Mutex<Connection>,
    app_data_dir: PathBuf,
}

fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_favorites(state: tauri::State<DbState>, data: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO state (key, value) VALUES ('favorites', ?1)",
        rusqlite::params![data],
    ).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn load_favorites(state: tauri::State<DbState>) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM state WHERE key = 'favorites'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_playlist(state: tauri::State<DbState>, data: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let len = data.len();
    let preview: String = data.chars().take(300).collect();
    conn.execute(
        "INSERT OR REPLACE INTO state (key, value) VALUES ('playlist', ?1)",
        rusqlite::params![data],
    ).map_err(|e| e.to_string())?;
    // Force flush to disk
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
    eprintln!("[db] playlist SAVED ({} bytes) preview: {}", len, preview);
    Ok(true)
}

#[tauri::command]
fn load_playlist(state: tauri::State<DbState>) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM state WHERE key = 'playlist'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(v) => {
            let preview: String = v.chars().take(300).collect();
            eprintln!("[db] playlist LOADED ({} bytes) preview: {}", v.len(), preview);
            Ok(v)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            eprintln!("[db] no saved playlist found");
            Ok(String::new())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Copy an audio file into the app's persistent data directory.
#[tauri::command]
fn copy_file_to_data(state: tauri::State<DbState>, source: String) -> Result<String, String> {
    let src = PathBuf::from(&source);
    let filename = src
        .file_name()
        .ok_or_else(|| "invalid source path".to_string())?
        .to_string_lossy()
        .to_string();

    let mut dest_dir = state.app_data_dir.clone();
    dest_dir.push("audio");
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let dest = dest_dir.join(&filename);
    if !dest.exists() {
        fs::copy(&src, &dest).map_err(|e| format!("copy failed: {}", e))?;
        eprintln!("[file] copied {} → {}", source, dest.display());
    }
    Ok(dest.to_string_lossy().to_string())
}

/// Read a text file (e.g. .lrc lyrics) as a UTF-8 string.
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("read failed: {}", e))
}

/// Open Finder at the file's location (macOS) or Explorer (Windows).
#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Finder: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Explorer: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("failed to open file manager: {}", e))?;
        }
    }
    Ok(())
}

// ========== 波形生成 (symphonia) ==========

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Generate waveform peak data from an audio file.
/// Returns (peaks: Vec<f32>, sample_rate: u32, total_samples: u64, duration_secs: f64).
/// peaks are normalized to [0.0, 1.0].
#[tauri::command]
async fn generate_waveform(path: String, num_peaks: Option<usize>) -> Result<serde_json::Value, String> {
    let num_peaks = num_peaks.unwrap_or(2000);
    // Run CPU-heavy decoding on a blocking thread
    tokio::task::spawn_blocking(move || {
        do_generate_waveform(&path, num_peaks)
    })
    .await
    .map_err(|e| format!("waveform thread panic: {}", e))?
}

fn do_generate_waveform(path: &str, num_peaks: usize) -> Result<serde_json::Value, String> {
    let src = std::fs::File::open(path).map_err(|e| format!("open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("probe format: {}", e))?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "no default track".to_string())?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let _n_frames = track.codec_params.n_frames;

    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("create decoder: {}", e))?;

    // First pass: collect all audio samples to determine total length
    let mut all_samples: Vec<f32> = Vec::new();
    let mut total_frames: u64 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("read packet: {}", e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("decode: {}", e)),
        };

        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        let samples = sample_buf.samples();
        let num_channels = spec.channels.count();

        // Downmix to mono by averaging channels
        let frames_in_packet = samples.len() / num_channels;
        all_samples.reserve(frames_in_packet);
        for frame_idx in 0..frames_in_packet {
            let mut sum = 0.0f32;
            for ch in 0..num_channels {
                let idx = frame_idx * num_channels + ch;
                sum += samples[idx].abs();
            }
            all_samples.push(sum / num_channels as f32);
        }
        total_frames += frames_in_packet as u64;
    }

    if all_samples.is_empty() {
        return Err("no audio samples decoded".to_string());
    }

    // Compute peak envelope
    let num_peaks = num_peaks.min(all_samples.len());
    let window_size = all_samples.len() / num_peaks;
    let mut peaks = Vec::with_capacity(num_peaks);

    // Find global max for normalization
    let global_max = all_samples
        .iter()
        .fold(0.0f32, |acc, &x| if x > acc { x } else { acc });

    if global_max <= 0.0 {
        return Err("audio is silent".to_string());
    }

    for i in 0..num_peaks {
        let start = i * window_size;
        let end = if i == num_peaks - 1 {
            all_samples.len()
        } else {
            (i + 1) * window_size
        };
        let peak = all_samples[start..end]
            .iter()
            .fold(0.0f32, |acc, &x| if x > acc { x } else { acc });
        peaks.push((peak / global_max).clamp(0.0, 1.0));
    }

    let duration_secs = total_frames as f64 / sample_rate as f64;

    Ok(serde_json::json!({
        "peaks": peaks,
        "sampleRate": sample_rate,
        "totalFrames": total_frames,
        "durationSecs": duration_secs,
    }))
}

/// Extract audio metadata: codec, sample rate, bitrate, channels, bit depth, file size, duration.
#[tauri::command]
async fn get_audio_info(path: String) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || do_get_audio_info(&path))
        .await
        .map_err(|e| format!("audio info thread panic: {}", e))?
}

fn do_get_audio_info(path: &str) -> Result<serde_json::Value, String> {
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0);

    let src = std::fs::File::open(path).map_err(|e| format!("open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("probe format: {}", e))?;

    let format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "no default track".to_string())?;

    let params = &track.codec_params;
    let codec_id = params.codec;
    let sample_rate = params.sample_rate.unwrap_or(0);
    let channels = params
        .channels
        .map(|c| c.count() as u32)
        .unwrap_or(0);
    let bits_per_sample = params.bits_per_sample.unwrap_or(0);
    let n_frames = params.n_frames.unwrap_or(0);
    let duration_secs = if sample_rate > 0 && n_frames > 0 {
        n_frames as f64 / sample_rate as f64
    } else {
        0.0
    };

    // Bitrate: prefer from container, fallback to calculation
    let bitrate_bps = if duration_secs > 0.0 && file_size > 0 {
        // Estimate from file size (includes container overhead, tags, etc.)
        let raw_bps = (file_size as f64 * 8.0) / duration_secs;
        // For variable-bitrate formats like Vorbis, just use it as-is
        Some(raw_bps as u64)
    } else {
        None
    };

    // Codec name
    let codec_name: String = {
        use symphonia::core::codecs::{
            CODEC_TYPE_FLAC, CODEC_TYPE_MP3, CODEC_TYPE_AAC,
            CODEC_TYPE_ALAC, CODEC_TYPE_VORBIS, CODEC_TYPE_OPUS,
        };
        if codec_id == CODEC_TYPE_FLAC {
            "FLAC".to_string()
        } else if codec_id == CODEC_TYPE_MP3 {
            "MP3".to_string()
        } else if codec_id == CODEC_TYPE_AAC {
            "AAC".to_string()
        } else if codec_id == CODEC_TYPE_ALAC {
            "ALAC".to_string()
        } else if codec_id == CODEC_TYPE_VORBIS {
            "Vorbis".to_string()
        } else if codec_id == CODEC_TYPE_OPUS {
            "Opus".to_string()
        } else {
            // PCM or unknown — try file extension
            if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "wav" => "WAV".to_string(),
                    "aiff" | "aif" => "AIFF".to_string(),
                    _ => "PCM / ?".to_string(),
                }
            } else {
                "?".to_string()
            }
        }
    };

    // Format file size
    let size_str = if file_size >= 1_000_000_000 {
        format!("{:.2} GB", file_size as f64 / 1_000_000_000.0)
    } else if file_size >= 1_000_000 {
        format!("{:.2} MB", file_size as f64 / 1_000_000.0)
    } else if file_size >= 1_000 {
        format!("{:.1} KB", file_size as f64 / 1_000.0)
    } else {
        format!("{} B", file_size)
    };

    // Format duration
    let duration_str = if duration_secs > 0.0 {
        let m = (duration_secs / 60.0) as u64;
        let s = (duration_secs % 60.0) as u64;
        format!("{}:{:02}", m, s)
    } else {
        "—".to_string()
    };

    Ok(serde_json::json!({
        "codec": codec_name,
        "sampleRate": sample_rate,
        "sampleRateStr": if sample_rate > 0 { format!("{:.1} kHz", sample_rate as f64 / 1000.0) } else { "—".to_string() },
        "channels": channels,
        "channelsStr": match channels { 1 => "Mono".to_string(), 2 => "Stereo".to_string(), n if n > 2 => format!("{}ch", n), _ => "—".to_string() },
        "bitDepth": bits_per_sample,
        "bitDepthStr": if bits_per_sample > 0 { format!("{}-bit", bits_per_sample) } else { "—".to_string() },
        "bitrate": bitrate_bps.unwrap_or(0),
        "bitrateStr": if let Some(bps) = bitrate_bps { format!("{} kbps", bps / 1000) } else { "—".to_string() },
        "durationSecs": duration_secs,
        "durationStr": duration_str,
        "fileSize": file_size,
        "fileSizeStr": size_str,
    }))
}

// ========== 格式转换 (ffmpeg-sidecar) ==========

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::download::auto_download;

/// Convert an audio file to a target format.
/// Supported formats: mp3, flac, wav, aac (m4a), ogg
#[tauri::command]
async fn convert_audio(source: String, format: String, bitrate: Option<String>) -> Result<String, String> {
    tokio::task::spawn_blocking(move || do_convert_audio(&source, &format, bitrate.as_deref()))
        .await
        .map_err(|e| format!("convert thread panic: {}", e))?
}

fn do_convert_audio(source: &str, format: &str, bitrate: Option<&str>) -> Result<String, String> {
    // Ensure ffmpeg binary is available (auto-download on first use)
    auto_download().map_err(|e| format!("ffmpeg download: {}", e))?;

    let src_path = std::path::Path::new(source);
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let out_ext = match format {
        "mp3" => "mp3",
        "flac" => "flac",
        "wav" => "wav",
        "aac" | "m4a" => "m4a",
        "ogg" => "ogg",
        _ => return Err(format!("unsupported format: {}", format)),
    };

    let out_dir = src_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let out_path = out_dir.join(format!("{}.{}", stem, out_ext));
    let out_path_str = out_path.to_string_lossy().to_string();

    let mut cmd = FfmpegCommand::new();
    cmd.input(source)
       .output(&out_path_str)
       .overwrite()
       .args(["-vn"])  // strip video if any
       ;

    // Codec and quality settings
    match format {
        "mp3" => {
            cmd.args(["-c:a", "libmp3lame", "-b:a", bitrate.unwrap_or("320k")]);
        }
        "flac" => {
            cmd.args(["-c:a", "flac", "-compression_level", "8"]);
        }
        "wav" => {
            // default PCM
        }
        "aac" | "m4a" => {
            cmd.args(["-c:a", "aac", "-b:a", bitrate.unwrap_or("256k")]);
        }
        "ogg" => {
            cmd.args(["-c:a", "libvorbis", "-q:a", bitrate.unwrap_or("6")]);
        }
        _ => {}
    }

    cmd.spawn()
        .map_err(|e| format!("ffmpeg spawn: {}", e))?
        .wait()
        .map_err(|e| format!("ffmpeg wait: {}", e))?;

    eprintln!("[convert] {} → {}", source, out_path_str);

    Ok(out_path_str)
}

/// Analyze loudness using EBU R128 (loudnorm).
/// Returns integrated loudness, range, true peak, and recommended gain.
#[tauri::command]
async fn analyze_loudness(path: String) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || do_analyze_loudness(&path))
        .await
        .map_err(|e| format!("loudness thread panic: {}", e))?
}

fn do_analyze_loudness(path: &str) -> Result<serde_json::Value, String> {
    use ffmpeg_sidecar::event::FfmpegEvent;
    use ffmpeg_sidecar::command::FfmpegCommand;

    auto_download().map_err(|e| format!("ffmpeg download: {}", e))?;

    let mut stderr_lines = String::new();

    FfmpegCommand::new()
        .input(path)
        .args([
            "-af", "loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json",
            "-f", "null", "-",
        ])
        .output("/dev/null")
        .spawn()
        .map_err(|e| format!("ffmpeg spawn: {}", e))?
        .iter()
        .map_err(|e| format!("ffmpeg iter: {}", e))?
        .for_each(|event| {
            if let FfmpegEvent::Log(_level, msg) = event {
                stderr_lines.push_str(&msg);
                stderr_lines.push('\n');
            }
        });

    // Find the JSON block in stderr
    let json_start = stderr_lines.find('{');
    let json_end = stderr_lines.rfind('}');

    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &stderr_lines[start..=end];
        let parsed: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("parse loudnorm json: {}", e))?;

        let input_i = parsed["input_i"].as_str().unwrap_or("0");
        let input_tp = parsed["input_tp"].as_str().unwrap_or("0");
        let input_lra = parsed["input_lra"].as_str().unwrap_or("0");
        let input_thresh = parsed["input_thresh"].as_str().unwrap_or("0");
        let target_offset = parsed["target_offset"].as_str().unwrap_or("0");

        let offset: f64 = target_offset.parse().unwrap_or(0.0);

        Ok(serde_json::json!({
            "integrated": input_i,
            "range": input_lra,
            "truePeak": input_tp,
            "threshold": input_thresh,
            "offsetDb": format!("{:.1}", offset),
            "offsetNum": offset,
        }))
    } else {
        Err("loudnorm: no JSON output found".to_string())
    }
}

/// Trim an audio segment and save as a new file.
/// start_sec and end_sec are in seconds.
#[tauri::command]
async fn trim_audio(source: String, start_sec: f64, end_sec: f64) -> Result<String, String> {
    tokio::task::spawn_blocking(move || do_trim_audio(&source, start_sec, end_sec))
        .await
        .map_err(|e| format!("trim thread panic: {}", e))?
}

fn do_trim_audio(source: &str, start_sec: f64, end_sec: f64) -> Result<String, String> {
    auto_download().map_err(|e| format!("ffmpeg download: {}", e))?;

    let src_path = std::path::Path::new(source);
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("trimmed");

    let ext = src_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("mp3");

    let out_dir = src_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let duration = end_sec - start_sec;
    let out_name = format!("{}_{:.0}s.{}", stem, duration, ext);
    let out_path = out_dir.join(&out_name);
    let out_path_str = out_path.to_string_lossy().to_string();

    FfmpegCommand::new()
        .input(source)
        .args([
            "-ss", &format!("{:.3}", start_sec),
            "-t", &format!("{:.3}", duration),
            "-c", "copy",  // stream copy (fast, no re-encode)
            "-avoid_negative_ts", "make_zero",
        ])
        .output(&out_path_str)
        .overwrite()
        .spawn()
        .map_err(|e| format!("ffmpeg spawn: {}", e))?
        .wait()
        .map_err(|e| format!("ffmpeg wait: {}", e))?;

    eprintln!("[trim] {} → {}", source, out_path_str);
    Ok(out_path_str)
}

/// Extract embedded album art from an audio file.
/// Returns the path to the extracted image file, or null if no art found.
#[tauri::command]
fn extract_cover_art(state: tauri::State<DbState>, path: String) -> Result<Option<String>, String> {
    use symphonia::core::meta::StandardVisualKey;

    let src = std::fs::File::open(&path).map_err(|e| format!("open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let mut probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("probe format: {}", e))?;

    // Get metadata revisions
    let metadata = probed.format.metadata();
    let current = metadata.current();

    // Look for cover art in all metadata revisions
    let mut cover_data: Option<(Vec<u8>, String)> = None; // (bytes, extension)

    if let Some(rev) = current {
        for visual in rev.visuals() {
            // Prefer FrontCover, but accept any
            let is_cover = match visual.usage {
                Some(StandardVisualKey::FrontCover) => true,
                Some(StandardVisualKey::OtherIcon) => true,
                None => {
                    // Generic picture — likely a cover
                    cover_data.is_none()
                }
                _ => false,
            };

            if is_cover {
                let ext = match visual.media_type.as_str() {
                    "image/jpeg" | "image/jpg" => "jpg",
                    "image/png" => "png",
                    "image/bmp" => "bmp",
                    "image/gif" => "gif",
                    "image/webp" => "webp",
                    _ => "jpg", // default
                };
                cover_data = Some((visual.data.to_vec(), ext.to_string()));
                break; // Take first cover
            }
        }
    }

    match cover_data {
        Some((data, ext)) => {
            // Save to app data dir
            let mut dest_dir = state.app_data_dir.clone();
            dest_dir.push("covers");
            fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

            // Use hash of path as filename to avoid collisions
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            path.hash(&mut hasher);
            let hash = format!("{:016x}", hasher.finish());

            let dest = dest_dir.join(format!("{}.{}", hash, ext));
            fs::write(&dest, &data).map_err(|e| format!("write cover: {}", e))?;

            eprintln!("[cover] extracted → {}", dest.display());
            Ok(Some(dest.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("failed to resolve app data dir");
            fs::create_dir_all(&app_data_dir)
                .expect("failed to create app data dir");

            let db_path = app_data_dir.join("data.db");
            eprintln!("[db] path: {}", db_path.display());

            let conn = Connection::open(&db_path)
                .expect("failed to open database");
            init_db(&conn)
                .expect("failed to initialize database");

            app.manage(DbState {
                conn: Mutex::new(conn),
                app_data_dir,
            });

            // macOS: use overlay titlebar style for native shadows + rounded corners
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_title_bar_style(TitleBarStyle::Overlay);
                }
            }

            // Build custom system menu — music player native
            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                build_menu(&handle, "进入全屏")?;

                // 监听窗口事件：全屏切换 + 关闭拦截
                if let Some(window) = app.get_webview_window("main") {
                    let handle = app.handle().clone();
                    window.on_window_event(move |event| {
                        match event {
                            tauri::WindowEvent::CloseRequested { api, .. } => {
                                // 拦截关闭：最小化窗口（兼容所有 Tauri 2.x 版本）
                                api.prevent_close();
                                if let Some(w) = handle.get_webview_window("main") {
                                    let _ = w.minimize();
                                }
                            }
                            tauri::WindowEvent::Focused(focused) if *focused => {
                                if let Some(w) = handle.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            tauri::WindowEvent::Resized(_) => {
                                if let Some(w) = handle.get_webview_window("main") {
                                    let is_fs = w.is_fullscreen().unwrap_or(false);
                                    let _ = build_menu(&handle, if is_fs { "退出全屏" } else { "进入全屏" });
                                }
                            }
                            _ => {}
                        }
                    });
                }

                // 系统托盘
                let tray_menu = MenuBuilder::new(app.handle())
                    .item(&MenuItemBuilder::with_id("tray_play", "播放 / 暂停").build(app.handle())?)
                    .item(&MenuItemBuilder::with_id("tray_next", "下一首").build(app.handle())?)
                    .item(&MenuItemBuilder::with_id("tray_prev", "上一首").build(app.handle())?)
                    .separator()
                    .item(&MenuItemBuilder::with_id("tray_show", "显示窗口").build(app.handle())?)
                    .separator()
                    .item(&MenuItemBuilder::with_id("tray_quit", "退出 Hanono").build(app.handle())?)
                    .build()?;

                // 解码托盘图标
                let tray_icon_data = include_bytes!("../icons/tray-icon.png");
                let decoder = png::Decoder::new(std::io::Cursor::new(tray_icon_data));
                let mut reader = decoder.read_info().unwrap();
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                let tray_image = tauri::image::Image::new_owned(buf, info.width, info.height);
                let _tray = TrayIconBuilder::new()
                    .icon(tray_image)
                    .icon_as_template(true)
                    .menu(&tray_menu)
                    .on_menu_event(|app, event| {
                        let id = event.id().as_ref();
                        match id {
                            "tray_play" | "tray_next" | "tray_prev" => {
                                let action = match id {
                                    "tray_play" => "play_pause",
                                    "tray_next" => "next",
                                    "tray_prev" => "prev",
                                    _ => return,
                                };
                                let _ = app.emit("menu-action", action);
                            }
                            "tray_show" => {
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            "tray_quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                            let _ = tray.app_handle().emit("menu-action", "play_pause");
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                // Playback controls
                "play_pause" | "next" | "prev" | "shuffle" | "repeat"
                | "vol_up" | "vol_down" => {
                    let _ = app.emit("menu-action", id);
                }
                // Playlist
                "import" | "favorite" | "clear" => {
                    let _ = app.emit("menu-action", id);
                }
                // About
                "about" => {
                    let _ = app.emit("menu-action", "about");
                }
                // Window controls
                "minimize" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.minimize();
                    }
                }
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                "zoom" => {
                    if let Some(w) = app.get_webview_window("main") {
                        if w.is_maximized().unwrap_or(false) {
                            let _ = w.unmaximize();
                        } else {
                            let _ = w.maximize();
                        }
                    }
                }
                "toggle_fullscreen" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let is_fullscreen = w.is_fullscreen().unwrap_or(false);
                        let _ = w.set_fullscreen(!is_fullscreen);
                        let _ = build_menu(app, if !is_fullscreen { "退出全屏" } else { "进入全屏" });
                    }
                }
                "toggle_ontop" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let is_top = w.is_always_on_top().unwrap_or(false);
                        let _ = w.set_always_on_top(!is_top);
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![save_playlist, load_playlist, save_favorites, load_favorites, copy_file_to_data, read_text_file, reveal_in_finder, generate_waveform, get_audio_info, convert_audio, analyze_loudness, trim_audio, extract_cover_art])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {});
}
