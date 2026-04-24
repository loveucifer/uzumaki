use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::Result;
use napi_derive::napi;

#[cfg(feature = "ffmpeg-backend")]
use ffmpeg_next as ffmpeg;
#[cfg(feature = "ffmpeg-backend")]
use std::collections::VecDeque;

#[napi(object)]
pub struct MediaLoadOptions {
    pub source: String,
    pub enable_video: Option<bool>,
    pub enable_audio: Option<bool>,
}

#[napi(object)]
pub struct MediaPlayerOptions {
    pub backend: Option<String>,
}

#[napi(object)]
pub struct MediaSnapshot {
    pub source: Option<String>,
    pub is_loaded: bool,
    pub is_playing: bool,
    pub enable_video: bool,
    pub enable_audio: bool,
    pub current_time_ms: u32,
    pub duration_ms: u32,
    pub volume: f64,
    pub muted: bool,
}

#[napi(object)]
pub struct CodecSupport {
    pub video: Vec<String>,
    pub audio: Vec<String>,
}

#[napi(object)]
pub struct VideoFramePacket {
    pub pts_ms: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: String,
    pub bytes: Vec<u8>,
}

#[napi(object)]
pub struct AudioPacket {
    pub pts_ms: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub sample_format: String,
    pub bytes: Vec<u8>,
}

#[cfg(feature = "ffmpeg-backend")]
#[derive(Default, Clone)]
struct SharedPlaybackState {
    source: Option<String>,
    is_loaded: bool,
    is_playing: bool,
    enable_video: bool,
    enable_audio: bool,
    current_time_ms: u64,
    duration_ms: u64,
    volume: f64,
    muted: bool,
}

trait MediaBackend: Send {
    fn id(&self) -> &'static str;
    fn codec_support(&self) -> CodecSupport;
    fn load(&mut self, options: &MediaLoadOptions) -> Result<MediaSnapshot>;
    fn play(&mut self) -> Result<MediaSnapshot>;
    fn pause(&mut self) -> Result<MediaSnapshot>;
    fn stop(&mut self) -> Result<MediaSnapshot>;
    fn seek(&mut self, position_ms: u32) -> Result<MediaSnapshot>;
    fn set_volume(&mut self, volume: f64) -> Result<MediaSnapshot>;
    fn set_muted(&mut self, muted: bool) -> Result<MediaSnapshot>;
    fn snapshot(&self) -> Result<MediaSnapshot>;
    fn tick(&mut self, delta_ms: u32) -> Result<MediaSnapshot>;
    fn read_video_frame(&mut self) -> Result<Option<VideoFramePacket>>;
    fn read_audio_packet(&mut self) -> Result<Option<AudioPacket>>;
}

#[cfg(feature = "ffmpeg-backend")]
struct FfmpegBackend {
    state: SharedPlaybackState,
    input: Option<ffmpeg::format::context::Input>,
    video_stream_index: Option<usize>,
    audio_stream_index: Option<usize>,
    video_decoder: Option<ffmpeg::decoder::Video>,
    audio_decoder: Option<ffmpeg::decoder::Audio>,
    video_frames: VecDeque<VideoFramePacket>,
    audio_packets: VecDeque<AudioPacket>,
    eof_reached: bool,
}

#[cfg(feature = "ffmpeg-backend")]
impl FfmpegBackend {
    fn new() -> Result<Self> {
        ffmpeg::init()
            .map_err(|err| napi::Error::from_reason(format!("ffmpeg init failed: {err}")))?;

        Ok(Self {
            state: SharedPlaybackState {
                volume: 1.0,
                ..Default::default()
            },
            input: None,
            video_stream_index: None,
            audio_stream_index: None,
            video_decoder: None,
            audio_decoder: None,
            video_frames: VecDeque::new(),
            audio_packets: VecDeque::new(),
            eof_reached: false,
        })
    }

    fn decode_packet_budget(&mut self, packet_budget: usize) -> Result<()> {
        for _ in 0..packet_budget {
            if self.eof_reached {
                break;
            }
            if !self.decode_one_packet()? {
                break;
            }
        }
        Ok(())
    }

    fn decode_one_packet(&mut self) -> Result<bool> {
        let Some(input) = self.input.as_mut() else {
            return Ok(false);
        };

        let mut packets = input.packets();
        let Some((stream, packet)) = packets.next() else {
            self.eof_reached = true;
            self.flush_decoders();
            return Ok(false);
        };

        let stream_index = stream.index();

        if self.state.enable_video
            && self.video_stream_index == Some(stream_index)
            && let Some(decoder) = self.video_decoder.as_mut()
        {
            decoder.send_packet(&packet).map_err(|err| {
                napi::Error::from_reason(format!("ffmpeg video send_packet failed: {err}"))
            })?;

            let mut decoded = ffmpeg::frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                self.video_frames.push_back(VideoFramePacket {
                    pts_ms: self.state.current_time_ms.min(u32::MAX as u64) as u32,
                    width: decoded.width(),
                    height: decoded.height(),
                    pixel_format: format!("{:?}", decoded.format()).to_lowercase(),
                    bytes: decoded.data(0).to_vec(),
                });
            }
        }

        if self.state.enable_audio
            && self.audio_stream_index == Some(stream_index)
            && let Some(decoder) = self.audio_decoder.as_mut()
        {
            decoder.send_packet(&packet).map_err(|err| {
                napi::Error::from_reason(format!("ffmpeg audio send_packet failed: {err}"))
            })?;

            let mut decoded = ffmpeg::frame::Audio::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                self.audio_packets.push_back(AudioPacket {
                    pts_ms: self.state.current_time_ms.min(u32::MAX as u64) as u32,
                    sample_rate: decoded.rate(),
                    channels: decoded.channels().min(u8::MAX as u16) as u8,
                    sample_format: format!("{:?}", decoded.format()).to_lowercase(),
                    bytes: decoded.data(0).to_vec(),
                });
            }
        }

        Ok(true)
    }

    fn flush_decoders(&mut self) {
        if let Some(decoder) = self.video_decoder.as_mut() {
            let _ = decoder.send_eof();
            let mut decoded = ffmpeg::frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                self.video_frames.push_back(VideoFramePacket {
                    pts_ms: self.state.current_time_ms.min(u32::MAX as u64) as u32,
                    width: decoded.width(),
                    height: decoded.height(),
                    pixel_format: format!("{:?}", decoded.format()).to_lowercase(),
                    bytes: decoded.data(0).to_vec(),
                });
            }
        }

        if let Some(decoder) = self.audio_decoder.as_mut() {
            let _ = decoder.send_eof();
            let mut decoded = ffmpeg::frame::Audio::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                self.audio_packets.push_back(AudioPacket {
                    pts_ms: self.state.current_time_ms.min(u32::MAX as u64) as u32,
                    sample_rate: decoded.rate(),
                    channels: decoded.channels().min(u8::MAX as u16) as u8,
                    sample_format: format!("{:?}", decoded.format()).to_lowercase(),
                    bytes: decoded.data(0).to_vec(),
                });
            }
        }
    }
}

#[cfg(feature = "ffmpeg-backend")]
impl MediaBackend for FfmpegBackend {
    fn id(&self) -> &'static str {
        "ffmpeg"
    }

    fn codec_support(&self) -> CodecSupport {
        CodecSupport {
            video: vec![
                "h264".to_string(),
                "hevc".to_string(),
                "vp9".to_string(),
                "av1".to_string(),
            ],
            audio: vec![
                "aac".to_string(),
                "mp3".to_string(),
                "opus".to_string(),
                "flac".to_string(),
                "pcm".to_string(),
            ],
        }
    }

    fn load(&mut self, options: &MediaLoadOptions) -> Result<MediaSnapshot> {
        let input = ffmpeg::format::input(&options.source).map_err(|err| {
            napi::Error::from_reason(format!("ffmpeg failed to open source '{}': {err}", options.source))
        })?;

        let enable_video = options.enable_video.unwrap_or(true);
        let enable_audio = options.enable_audio.unwrap_or(true);

        let video_stream_index = if enable_video {
            input
                .streams()
                .best(ffmpeg::media::Type::Video)
                .map(|stream| stream.index())
        } else {
            None
        };

        let audio_stream_index = if enable_audio {
            input
                .streams()
                .best(ffmpeg::media::Type::Audio)
                .map(|stream| stream.index())
        } else {
            None
        };

        let video_decoder = if let Some(stream_index) = video_stream_index {
            let stream = input.stream(stream_index).ok_or_else(|| {
                napi::Error::from_reason("video stream index resolved but stream not found")
            })?;
            let codec_ctx = ffmpeg::codec::Context::from_parameters(stream.parameters()).map_err(
                |err| napi::Error::from_reason(format!("ffmpeg video context error: {err}")),
            )?;
            Some(codec_ctx.decoder().video().map_err(|err| {
                napi::Error::from_reason(format!("ffmpeg video decoder error: {err}"))
            })?)
        } else {
            None
        };

        let audio_decoder = if let Some(stream_index) = audio_stream_index {
            let stream = input.stream(stream_index).ok_or_else(|| {
                napi::Error::from_reason("audio stream index resolved but stream not found")
            })?;
            let codec_ctx = ffmpeg::codec::Context::from_parameters(stream.parameters()).map_err(
                |err| napi::Error::from_reason(format!("ffmpeg audio context error: {err}")),
            )?;
            Some(codec_ctx.decoder().audio().map_err(|err| {
                napi::Error::from_reason(format!("ffmpeg audio decoder error: {err}"))
            })?)
        } else {
            None
        };

        self.state.source = Some(options.source.clone());
        self.state.enable_video = enable_video;
        self.state.enable_audio = enable_audio;
        self.state.current_time_ms = 0;
        self.state.duration_ms = input.duration().max(0) as u64 / 1000;
        self.state.is_loaded = true;
        self.state.is_playing = false;

        self.video_stream_index = video_stream_index;
        self.audio_stream_index = audio_stream_index;
        self.video_decoder = video_decoder;
        self.audio_decoder = audio_decoder;
        self.input = Some(input);
        self.video_frames.clear();
        self.audio_packets.clear();
        self.eof_reached = false;

        Ok(snapshot_from_state(&self.state))
    }

    fn play(&mut self) -> Result<MediaSnapshot> {
        ensure_loaded(&self.state)?;
        self.state.is_playing = true;
        Ok(snapshot_from_state(&self.state))
    }

    fn pause(&mut self) -> Result<MediaSnapshot> {
        ensure_loaded(&self.state)?;
        self.state.is_playing = false;
        Ok(snapshot_from_state(&self.state))
    }

    fn stop(&mut self) -> Result<MediaSnapshot> {
        ensure_loaded(&self.state)?;
        self.state.is_playing = false;
        self.state.current_time_ms = 0;
        self.video_frames.clear();
        self.audio_packets.clear();
        Ok(snapshot_from_state(&self.state))
    }

    fn seek(&mut self, position_ms: u32) -> Result<MediaSnapshot> {
        ensure_loaded(&self.state)?;

        if let Some(input) = self.input.as_mut() {
            let target_us = position_ms as i64 * 1000;
            let _ = input.seek(target_us, ..);
        }

        self.state.current_time_ms = position_ms as u64;
        self.video_frames.clear();
        self.audio_packets.clear();
        self.eof_reached = false;
        Ok(snapshot_from_state(&self.state))
    }

    fn set_volume(&mut self, volume: f64) -> Result<MediaSnapshot> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(napi::Error::from_reason(
                "volume must be in range 0.0..=1.0",
            ));
        }
        self.state.volume = volume;
        Ok(snapshot_from_state(&self.state))
    }

    fn set_muted(&mut self, muted: bool) -> Result<MediaSnapshot> {
        self.state.muted = muted;
        Ok(snapshot_from_state(&self.state))
    }

    fn snapshot(&self) -> Result<MediaSnapshot> {
        Ok(snapshot_from_state(&self.state))
    }

    fn tick(&mut self, delta_ms: u32) -> Result<MediaSnapshot> {
        if self.state.is_loaded && self.state.is_playing {
            self.state.current_time_ms = self.state.current_time_ms.saturating_add(delta_ms as u64);
            self.decode_packet_budget(24)?;

            if self.state.duration_ms > 0 && self.state.current_time_ms >= self.state.duration_ms {
                self.state.current_time_ms = self.state.duration_ms;
                self.state.is_playing = false;
            }

            if self.eof_reached && self.video_frames.is_empty() && self.audio_packets.is_empty() {
                self.state.is_playing = false;
            }
        }

        Ok(snapshot_from_state(&self.state))
    }

    fn read_video_frame(&mut self) -> Result<Option<VideoFramePacket>> {
        Ok(self.video_frames.pop_front())
    }

    fn read_audio_packet(&mut self) -> Result<Option<AudioPacket>> {
        Ok(self.audio_packets.pop_front())
    }
}

struct MediaCore {
    backend: Box<dyn MediaBackend>,
}

impl MediaCore {
    fn new(backend_name: Option<&str>) -> Result<Self> {
        let selected = if let Some(name) = backend_name {
            name
        } else {
            default_backend_name().ok_or_else(|| {
                napi::Error::from_reason(
                    "no media backend available in this build; compile with ffmpeg-backend feature",
                )
            })?
        };

        Ok(Self {
            backend: create_backend(selected)?,
        })
    }

    fn set_backend(&mut self, backend_name: &str) -> Result<String> {
        self.backend = create_backend(backend_name)?;
        Ok(self.backend.id().to_string())
    }
}

#[napi]
pub struct MediaPlayer {
    core: Arc<Mutex<MediaCore>>,
}

#[napi]
impl MediaPlayer {
    #[napi(constructor)]
    pub fn new(options: Option<MediaPlayerOptions>) -> Result<Self> {
        let backend_name = options.and_then(|it| it.backend);
        let core = MediaCore::new(backend_name.as_deref())?;
        Ok(Self {
            core: Arc::new(Mutex::new(core)),
        })
    }

    #[napi]
    pub fn backend(&self) -> Result<String> {
        let core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        Ok(core.backend.id().to_string())
    }

    #[napi]
    pub fn set_backend(&self, backend: String) -> Result<String> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.set_backend(&backend)
    }

    #[napi]
    pub fn load(&self, options: MediaLoadOptions) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.load(&options)
    }

    #[napi]
    pub fn play(&self) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.play()
    }

    #[napi]
    pub fn pause(&self) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.pause()
    }

    #[napi]
    pub fn stop(&self) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.stop()
    }

    #[napi]
    pub fn seek(&self, position_ms: u32) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.seek(position_ms)
    }

    #[napi]
    pub fn set_volume(&self, volume: f64) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.set_volume(volume)
    }

    #[napi]
    pub fn set_muted(&self, muted: bool) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.set_muted(muted)
    }

    #[napi]
    pub fn snapshot(&self) -> Result<MediaSnapshot> {
        let core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.snapshot()
    }

    #[napi]
    pub fn tick(&self, delta_ms: u32) -> Result<MediaSnapshot> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.tick(delta_ms)
    }

    #[napi]
    pub fn read_video_frame(&self) -> Result<Option<VideoFramePacket>> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.read_video_frame()
    }

    #[napi]
    pub fn read_audio_packet(&self) -> Result<Option<AudioPacket>> {
        let mut core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        core.backend.read_audio_packet()
    }

    #[napi]
    pub fn codec_support(&self) -> Result<CodecSupport> {
        let core = self
            .core
            .lock()
            .map_err(|_| napi::Error::from_reason("media core lock poisoned"))?;
        Ok(core.backend.codec_support())
    }
}

#[napi]
pub fn get_codec_support() -> CodecSupport {
    if let Some(default_backend) = default_backend_name()
        && let Ok(backend) = create_backend(default_backend)
    {
        return backend.codec_support();
    }

    CodecSupport {
        video: Vec::new(),
        audio: Vec::new(),
    }
}

#[napi]
pub fn supports_video_codec(codec: String) -> bool {
    get_codec_support().video.iter().any(|c| c == &codec)
}

#[napi]
pub fn supports_audio_codec(codec: String) -> bool {
    get_codec_support().audio.iter().any(|c| c == &codec)
}

#[napi]
pub fn available_media_backends() -> Vec<String> {
    available_backend_names()
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

#[cfg(feature = "ffmpeg-backend")]
fn create_backend(name: &str) -> Result<Box<dyn MediaBackend>> {
    match name {
        "ffmpeg" => Ok(Box::new(FfmpegBackend::new()?)),
        _ => Err(napi::Error::from_reason(format!(
            "unknown media backend '{name}', available: {}",
            available_backend_names().join(", ")
        ))),
    }
}

#[cfg(not(feature = "ffmpeg-backend"))]
fn create_backend(name: &str) -> Result<Box<dyn MediaBackend>> {
    if name == "ffmpeg" {
        return Err(napi::Error::from_reason(
            "ffmpeg backend is not compiled in; enable the ffmpeg-backend feature",
        ));
    }

    Err(napi::Error::from_reason(format!(
        "unknown media backend '{name}', available: {}",
        available_backend_names().join(", ")
    )))
}

#[cfg(feature = "ffmpeg-backend")]
fn available_backend_names() -> Vec<&'static str> {
    vec!["ffmpeg"]
}

#[cfg(not(feature = "ffmpeg-backend"))]
fn available_backend_names() -> Vec<&'static str> {
    Vec::new()
}

#[cfg(feature = "ffmpeg-backend")]
fn default_backend_name() -> Option<&'static str> {
    Some("ffmpeg")
}

#[cfg(not(feature = "ffmpeg-backend"))]
fn default_backend_name() -> Option<&'static str> {
    None
}

#[cfg(feature = "ffmpeg-backend")]
fn ensure_loaded(state: &SharedPlaybackState) -> Result<()> {
    if !state.is_loaded {
        return Err(napi::Error::from_reason(
            "media source is not loaded; call load() first",
        ));
    }
    Ok(())
}

#[cfg(feature = "ffmpeg-backend")]
fn snapshot_from_state(state: &SharedPlaybackState) -> MediaSnapshot {
    MediaSnapshot {
        source: state.source.clone(),
        is_loaded: state.is_loaded,
        is_playing: state.is_playing,
        enable_video: state.enable_video,
        enable_audio: state.enable_audio,
        current_time_ms: state.current_time_ms.min(u32::MAX as u64) as u32,
        duration_ms: state.duration_ms.min(u32::MAX as u64) as u32,
        volume: state.volume,
        muted: state.muted,
    }
}
