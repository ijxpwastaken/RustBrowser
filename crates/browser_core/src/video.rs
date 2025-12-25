//! Video Player Module
//!
//! Provides video decoding and playback functionality for the browser.
//! Uses a software-based decoder for maximum compatibility.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Video frame in RGBA format
#[derive(Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,  // RGBA pixels
    pub pts: f64,       // Presentation timestamp in seconds
    pub duration: f64,  // Frame duration in seconds
}

/// Video player state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Ended,
    Error,
}

/// Video metadata
#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration: f64,          // Total duration in seconds
    pub frame_rate: f64,        // Frames per second
    pub has_audio: bool,
    pub codec: String,
}

/// Video player for decoding and playing videos
pub struct VideoPlayer {
    /// Current playback state
    state: PlaybackState,
    
    /// Video metadata
    metadata: Option<VideoMetadata>,
    
    /// Decoded frame buffer
    frame_buffer: VecDeque<VideoFrame>,
    
    /// Current frame
    current_frame: Option<VideoFrame>,
    
    /// Playback start time
    playback_start: Option<Instant>,
    
    /// Playback position offset (for seeking/pause)
    position_offset: f64,
    
    /// Volume (0.0 to 1.0)
    volume: f32,
    
    /// Muted
    muted: bool,
    
    /// Loop playback
    looping: bool,
    
    /// Video source URL
    source_url: Option<String>,
    
    /// Max buffer size
    max_buffer_size: usize,
}

impl VideoPlayer {
    /// Create a new video player
    pub fn new() -> Self {
        VideoPlayer {
            state: PlaybackState::Stopped,
            metadata: None,
            frame_buffer: VecDeque::new(),
            current_frame: None,
            playback_start: None,
            position_offset: 0.0,
            volume: 1.0,
            muted: false,
            looping: false,
            source_url: None,
            max_buffer_size: 60, // Buffer up to 60 frames (about 2 seconds at 30fps)
        }
    }
    
    /// Load video from URL
    pub fn load(&mut self, url: &str) {
        self.source_url = Some(url.to_string());
        self.state = PlaybackState::Buffering;
        self.frame_buffer.clear();
        self.current_frame = None;
        self.position_offset = 0.0;
        self.playback_start = None;
        
        // For now, create placeholder metadata
        // In a real implementation, this would parse video headers
        self.metadata = Some(VideoMetadata {
            width: 1280,
            height: 720,
            duration: 0.0,
            frame_rate: 30.0,
            has_audio: true,
            codec: "h264".to_string(),
        });
        
        println!("[VideoPlayer] Loading video: {}", url);
    }
    
    /// Start playback
    pub fn play(&mut self) {
        match self.state {
            PlaybackState::Stopped | PlaybackState::Paused | PlaybackState::Ended => {
                if self.state == PlaybackState::Ended && self.looping {
                    self.position_offset = 0.0;
                }
                self.playback_start = Some(Instant::now());
                self.state = PlaybackState::Playing;
                println!("[VideoPlayer] Playing");
            }
            _ => {}
        }
    }
    
    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            // Save current position
            if let Some(start) = self.playback_start {
                self.position_offset += start.elapsed().as_secs_f64();
            }
            self.playback_start = None;
            self.state = PlaybackState::Paused;
            println!("[VideoPlayer] Paused");
        }
    }
    
    /// Stop playback
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.playback_start = None;
        self.position_offset = 0.0;
        self.current_frame = None;
        println!("[VideoPlayer] Stopped");
    }
    
    /// Seek to position (in seconds)
    pub fn seek(&mut self, position: f64) {
        self.position_offset = position.max(0.0);
        if let Some(ref meta) = self.metadata {
            if position >= meta.duration && meta.duration > 0.0 {
                self.position_offset = meta.duration;
            }
        }
        self.playback_start = if self.state == PlaybackState::Playing {
            Some(Instant::now())
        } else {
            None
        };
        self.frame_buffer.clear();
        self.current_frame = None;
        println!("[VideoPlayer] Seeking to: {:.2}s", position);
    }
    
    /// Get current playback position
    pub fn get_position(&self) -> f64 {
        let elapsed = match self.playback_start {
            Some(start) => start.elapsed().as_secs_f64(),
            None => 0.0,
        };
        self.position_offset + elapsed
    }
    
    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
    
    /// Get current volume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    
    /// Set muted state
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }
    
    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }
    
    /// Set loop state
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
    
    /// Get playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state
    }
    
    /// Get video metadata
    pub fn get_metadata(&self) -> Option<&VideoMetadata> {
        self.metadata.as_ref()
    }
    
    /// Add a decoded frame to the buffer
    pub fn add_frame(&mut self, frame: VideoFrame) {
        if self.frame_buffer.len() < self.max_buffer_size {
            self.frame_buffer.push_back(frame);
        }
    }
    
    /// Get the current frame for rendering
    pub fn get_current_frame(&mut self) -> Option<&VideoFrame> {
        if self.state != PlaybackState::Playing {
            return self.current_frame.as_ref();
        }
        
        let current_time = self.get_position();
        
        // Find the frame that should be displayed
        while let Some(frame) = self.frame_buffer.front() {
            if frame.pts + frame.duration < current_time {
                // This frame is in the past, remove it
                self.current_frame = self.frame_buffer.pop_front();
            } else if frame.pts <= current_time {
                // This is the current frame
                self.current_frame = self.frame_buffer.pop_front();
                break;
            } else {
                // Frame is in the future, wait
                break;
            }
        }
        
        // Check for end of video
        if self.frame_buffer.is_empty() && self.current_frame.is_none() {
            if let Some(ref meta) = self.metadata {
                if current_time >= meta.duration && meta.duration > 0.0 {
                    if self.looping {
                        self.seek(0.0);
                    } else {
                        self.state = PlaybackState::Ended;
                    }
                }
            }
        }
        
        self.current_frame.as_ref()
    }
    
    /// Decode a test pattern frame for demonstration
    pub fn generate_test_frame(&self, time: f64, width: u32, height: u32) -> VideoFrame {
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        
        // Generate a colorful test pattern that animates
        let phase = (time * 2.0) % 6.28;
        
        for y in 0..height {
            for x in 0..width {
                let fx = x as f64 / width as f64;
                let fy = y as f64 / height as f64;
                
                // Color bars with animation
                let r = ((fx + phase / 6.28).sin() * 127.0 + 128.0) as u8;
                let g = ((fy + phase / 4.0).cos() * 127.0 + 128.0) as u8;
                let b = ((fx * 3.14 + fy * 3.14 + phase).sin() * 127.0 + 128.0) as u8;
                
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(255);
            }
        }
        
        VideoFrame {
            width,
            height,
            data,
            pts: time,
            duration: 1.0 / 30.0,
        }
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared video player for thread-safe access
pub type SharedVideoPlayer = Arc<Mutex<VideoPlayer>>;

/// Create a shared video player
pub fn create_shared_player() -> SharedVideoPlayer {
    Arc::new(Mutex::new(VideoPlayer::new()))
}

// ============================================================================
// VIDEO DECODER TRAIT
// ============================================================================

/// Trait for video decoders
pub trait VideoDecoder: Send {
    /// Decode video from bytes
    fn decode(&mut self, data: &[u8]) -> Result<Vec<VideoFrame>, VideoError>;
    
    /// Get metadata from video
    fn get_metadata(&self) -> Option<&VideoMetadata>;
    
    /// Seek to position
    fn seek(&mut self, position: f64) -> Result<(), VideoError>;
    
    /// Check if decoder supports format
    fn supports_format(&self, format: &str) -> bool;
}

/// Video decoding errors
#[derive(Debug, Clone)]
pub enum VideoError {
    UnsupportedFormat(String),
    DecodingError(String),
    IoError(String),
    InvalidData(String),
}

impl std::fmt::Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoError::UnsupportedFormat(s) => write!(f, "Unsupported format: {}", s),
            VideoError::DecodingError(s) => write!(f, "Decoding error: {}", s),
            VideoError::IoError(s) => write!(f, "IO error: {}", s),
            VideoError::InvalidData(s) => write!(f, "Invalid data: {}", s),
        }
    }
}

impl std::error::Error for VideoError {}

// ============================================================================
// SIMPLE GIF/WEBP DECODER
// ============================================================================

/// Simple image sequence decoder for GIF/WebP animations
pub struct SimpleAnimationDecoder {
    frames: Vec<VideoFrame>,
    metadata: Option<VideoMetadata>,
    current_index: usize,
}

impl SimpleAnimationDecoder {
    pub fn new() -> Self {
        SimpleAnimationDecoder {
            frames: Vec::new(),
            metadata: None,
            current_index: 0,
        }
    }
    
    /// Decode from image bytes (simplified - would use image crate)
    pub fn decode_gif(&mut self, data: &[u8]) -> Result<(), VideoError> {
        // This is a placeholder - in a real implementation, 
        // we would use the `image` crate or `gif` crate to decode
        
        if data.len() < 6 || &data[0..6] != b"GIF89a" && &data[0..6] != b"GIF87a" {
            return Err(VideoError::InvalidData("Not a valid GIF".to_string()));
        }
        
        // For now, create a placeholder frame
        let frame = VideoFrame {
            width: 100,
            height: 100,
            data: vec![128u8; 100 * 100 * 4], // Gray placeholder
            pts: 0.0,
            duration: 0.1,
        };
        self.frames.push(frame);
        
        self.metadata = Some(VideoMetadata {
            width: 100,
            height: 100,
            duration: 0.1,
            frame_rate: 10.0,
            has_audio: false,
            codec: "gif".to_string(),
        });
        
        Ok(())
    }
    
    /// Get next frame
    pub fn next_frame(&mut self) -> Option<VideoFrame> {
        if self.frames.is_empty() {
            return None;
        }
        
        let frame = self.frames[self.current_index].clone();
        self.current_index = (self.current_index + 1) % self.frames.len();
        Some(frame)
    }
}

impl Default for SimpleAnimationDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoDecoder for SimpleAnimationDecoder {
    fn decode(&mut self, data: &[u8]) -> Result<Vec<VideoFrame>, VideoError> {
        self.decode_gif(data)?;
        Ok(self.frames.clone())
    }
    
    fn get_metadata(&self) -> Option<&VideoMetadata> {
        self.metadata.as_ref()
    }
    
    fn seek(&mut self, position: f64) -> Result<(), VideoError> {
        if let Some(ref meta) = self.metadata {
            if meta.frame_rate > 0.0 {
                let frame_index = (position * meta.frame_rate) as usize;
                self.current_index = frame_index % self.frames.len().max(1);
            }
        }
        Ok(())
    }
    
    fn supports_format(&self, format: &str) -> bool {
        matches!(format.to_lowercase().as_str(), "gif" | "webp" | "apng")
    }
}

// ============================================================================
// HTML5 VIDEO ELEMENT SUPPORT
// ============================================================================

/// HTML5 Video element representation
pub struct HtmlVideoElement {
    pub player: VideoPlayer,
    pub src: String,
    pub poster: Option<String>,
    pub autoplay: bool,
    pub controls: bool,
    pub loop_playback: bool,
    pub muted: bool,
    pub preload: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl HtmlVideoElement {
    pub fn new() -> Self {
        HtmlVideoElement {
            player: VideoPlayer::new(),
            src: String::new(),
            poster: None,
            autoplay: false,
            controls: true,
            loop_playback: false,
            muted: false,
            preload: "auto".to_string(),
            width: None,
            height: None,
        }
    }
    
    /// Set video source
    pub fn set_src(&mut self, src: &str) {
        self.src = src.to_string();
        self.player.load(src);
        if self.autoplay {
            self.player.play();
        }
    }
    
    /// JavaScript interface: play()
    pub fn js_play(&mut self) {
        self.player.play();
    }
    
    /// JavaScript interface: pause()
    pub fn js_pause(&mut self) {
        self.player.pause();
    }
    
    /// JavaScript interface: currentTime getter
    pub fn get_current_time(&self) -> f64 {
        self.player.get_position()
    }
    
    /// JavaScript interface: currentTime setter
    pub fn set_current_time(&mut self, time: f64) {
        self.player.seek(time);
    }
    
    /// JavaScript interface: duration getter
    pub fn get_duration(&self) -> f64 {
        self.player.get_metadata()
            .map(|m| m.duration)
            .unwrap_or(0.0)
    }
    
    /// JavaScript interface: paused getter
    pub fn is_paused(&self) -> bool {
        self.player.get_state() != PlaybackState::Playing
    }
    
    /// JavaScript interface: ended getter
    pub fn is_ended(&self) -> bool {
        self.player.get_state() == PlaybackState::Ended
    }
    
    /// JavaScript interface: volume getter/setter
    pub fn get_volume(&self) -> f32 {
        self.player.get_volume()
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
    }
    
    /// JavaScript interface: muted getter/setter
    pub fn get_muted(&self) -> bool {
        self.player.is_muted()
    }
    
    pub fn set_muted(&mut self, muted: bool) {
        self.player.set_muted(muted);
    }
    
    /// Render video frame to RGBA buffer
    pub fn render_to_buffer(&mut self) -> Option<&VideoFrame> {
        self.player.get_current_frame()
    }
}

impl Default for HtmlVideoElement {
    fn default() -> Self {
        Self::new()
    }
}
