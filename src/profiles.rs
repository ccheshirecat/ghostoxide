//! Stealth profile system for customizable browser fingerprints.
//!
//! This module provides an ergonomic builder pattern for creating consistent
//! browser "personalities" that bypass anti-bot detection.
//!
//! # Example
//!
//! ```rust
//! use chaser-oxide::profiles::{ChaserProfile, Gpu};
//!
//! let profile = ChaserProfile::windows()
//!     .chrome_version(130)
//!     .gpu(Gpu::NvidiaRTX4080)
//!     .memory_gb(16)
//!     .cpu_cores(12)
//!     .build();
//! ```

use std::fmt;

/// GPU presets for WebGL spoofing
#[derive(Debug, Clone, Copy)]
pub enum Gpu {
    /// NVIDIA GeForce RTX 3080 (high-trust gaming GPU)
    NvidiaRTX3080,
    /// NVIDIA GeForce RTX 4080 (newer gaming GPU)
    NvidiaRTX4080,
    /// NVIDIA GeForce GTX 1660 (mid-range GPU)
    NvidiaGTX1660,
    /// Intel UHD Graphics 630 (common laptop GPU)
    IntelUHD630,
    /// Intel Iris Xe (modern laptop GPU)
    IntelIrisXe,
    /// Apple M1 Pro
    AppleM1Pro,
    /// Apple M2 Max
    AppleM2Max,
    /// Apple M4 Max
    AppleM4Max,
    /// AMD Radeon RX 6800
    AmdRadeonRX6800,
}

impl Gpu {
    /// Returns the WebGL vendor string
    pub fn vendor(&self) -> &'static str {
        match self {
            Gpu::NvidiaRTX3080 | Gpu::NvidiaRTX4080 | Gpu::NvidiaGTX1660 => "Google Inc. (NVIDIA)",
            Gpu::IntelUHD630 | Gpu::IntelIrisXe => "Google Inc. (Intel)",
            Gpu::AppleM1Pro | Gpu::AppleM2Max | Gpu::AppleM4Max => "Google Inc. (Apple)",
            Gpu::AmdRadeonRX6800 => "Google Inc. (AMD)",
        }
    }

    /// Returns the WebGL renderer string
    pub fn renderer(&self) -> &'static str {
        match self {
            Gpu::NvidiaRTX3080 => {
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::NvidiaRTX4080 => {
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::NvidiaGTX1660 => {
                "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 SUPER Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::IntelUHD630 => "ANGLE (Intel, Intel(R) UHD Graphics 630 Direct3D11 vs_5_0 ps_5_0)",
            Gpu::IntelIrisXe => {
                "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0)"
            }
            Gpu::AppleM1Pro => "ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)",
            Gpu::AppleM2Max => "ANGLE (Apple, Apple M2 Max, OpenGL 4.1)",
            Gpu::AppleM4Max => {
                "ANGLE (Apple, ANGLE Metal Renderer: Apple M4 Max, Unspecified Version)"
            }
            Gpu::AmdRadeonRX6800 => "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0)",
        }
    }
}

/// Operating system presets
#[derive(Debug, Clone, Copy)]
pub enum Os {
    /// Windows 10/11 64-bit
    Windows,
    /// macOS (Intel)
    MacOSIntel,
    /// macOS (Apple Silicon)
    MacOSArm,
    /// Linux x86_64
    Linux,
}

impl Os {
    /// Returns the navigator.platform value
    pub fn platform(&self) -> &'static str {
        match self {
            Os::Windows => "Win32",
            Os::MacOSIntel | Os::MacOSArm => "MacIntel",
            Os::Linux => "Linux x86_64",
        }
    }

    /// Returns the client hints platform
    pub fn hints_platform(&self) -> &'static str {
        match self {
            Os::Windows => "Windows",
            Os::MacOSIntel | Os::MacOSArm => "macOS",
            Os::Linux => "Linux",
        }
    }
}

/// A builder for creating consistent browser fingerprint profiles.
///
/// # Example
///
/// ```rust
/// use chaser-oxide::profiles::{ChaserProfile, Gpu, Os};
///
/// // Quick preset
/// let profile = ChaserProfile::windows().build();
///
/// // Customized
/// let profile = ChaserProfile::new(Os::Windows)
///     .chrome_version(130)
///     .gpu(Gpu::NvidiaRTX4080)
///     .memory_gb(32)
///     .cpu_cores(16)
///     .locale("de-DE")
///     .timezone("Europe/Berlin")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ChaserProfile {
    os: Os,
    chrome_version: u32,
    gpu: Gpu,
    memory_gb: u32,
    cpu_cores: u32,
    locale: String,
    timezone: String,
    screen_width: u32,
    screen_height: u32,
}

impl Default for ChaserProfile {
    fn default() -> Self {
        Self::windows().build()
    }
}

impl ChaserProfile {
    /// Create a new profile builder with the specified OS
    #[allow(clippy::new_ret_no_self)]
    pub fn new(os: Os) -> ChaserProfileBuilder {
        ChaserProfileBuilder {
            os,
            chrome_version: 129,
            gpu: match os {
                Os::Windows => Gpu::NvidiaRTX3080,
                Os::MacOSIntel => Gpu::AppleM1Pro,
                Os::MacOSArm => Gpu::AppleM4Max,
                Os::Linux => Gpu::NvidiaGTX1660,
            },
            memory_gb: 8,
            cpu_cores: 8,
            locale: "en-US".to_string(),
            timezone: "America/New_York".to_string(),
            screen_width: 1920,
            screen_height: 1080,
        }
    }

    /// Create a Windows profile with sensible defaults (RTX 3080, 8 cores)
    pub fn windows() -> ChaserProfileBuilder {
        Self::new(Os::Windows)
    }

    /// Create a macOS Intel profile
    pub fn macos_intel() -> ChaserProfileBuilder {
        Self::new(Os::MacOSIntel).gpu(Gpu::AppleM1Pro)
    }

    /// Create a macOS Apple Silicon profile
    pub fn macos_arm() -> ChaserProfileBuilder {
        Self::new(Os::MacOSArm).gpu(Gpu::AppleM4Max)
    }

    /// Create a Linux profile
    pub fn linux() -> ChaserProfileBuilder {
        Self::new(Os::Linux)
    }

    // Getters
    pub fn os(&self) -> Os {
        self.os
    }
    pub fn chrome_version(&self) -> u32 {
        self.chrome_version
    }
    pub fn gpu(&self) -> Gpu {
        self.gpu
    }
    pub fn memory_gb(&self) -> u32 {
        self.memory_gb
    }
    pub fn cpu_cores(&self) -> u32 {
        self.cpu_cores
    }
    pub fn locale(&self) -> &str {
        &self.locale
    }
    pub fn timezone(&self) -> &str {
        &self.timezone
    }
    pub fn screen_width(&self) -> u32 {
        self.screen_width
    }
    pub fn screen_height(&self) -> u32 {
        self.screen_height
    }

    /// Generate the User-Agent string for this profile
    pub fn user_agent(&self) -> String {
        let os_part = match self.os {
            Os::Windows => "Windows NT 10.0; Win64; x64",
            Os::MacOSIntel | Os::MacOSArm => "Macintosh; Intel Mac OS X 10_15_7",
            Os::Linux => "X11; Linux x86_64",
        };
        format!(
            "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
            os_part, self.chrome_version
        )
    }

    /// Generate the complete JavaScript bootstrap script for this profile
    pub fn bootstrap_script(&self) -> String {
        format!(
            r#"
            (function() {{
                // === chaser-oxide HARDWARE HARMONY ===
                // Profile: {ua}

                // 1. Platform
                Object.defineProperty(navigator, 'platform', {{
                    get: () => '{platform}',
                    configurable: true
                }});

                // 2. Hardware
                Object.defineProperty(navigator, 'hardwareConcurrency', {{
                    get: () => {cores},
                    configurable: true
                }});
                Object.defineProperty(navigator, 'deviceMemory', {{
                    get: () => {memory},
                    configurable: true
                }});
                Object.defineProperty(navigator, 'maxTouchPoints', {{
                    get: () => 0,
                    configurable: true
                }});

                // 3. WebGL
                const spoofWebGL = (proto) => {{
                    const getParameter = proto.getParameter;
                    proto.getParameter = function(parameter) {{
                        if (parameter === 37445) return '{webgl_vendor}';
                        if (parameter === 37446) return '{webgl_renderer}';
                        return getParameter.apply(this, arguments);
                    }};
                }};
                spoofWebGL(WebGLRenderingContext.prototype);
                if (typeof WebGL2RenderingContext !== 'undefined') {{
                    spoofWebGL(WebGL2RenderingContext.prototype);
                }}

                // 4. Client Hints
                Object.defineProperty(navigator, 'userAgentData', {{
                    get: () => ({{
                        brands: [
                            {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                            {{ brand: "Chromium", version: "{chrome_ver}" }},
                            {{ brand: "Not=A?Brand", version: "24" }}
                        ],
                        mobile: false,
                        platform: "{hints_platform}"
                    }})
                }});

                // 5. Video Codecs
                const canPlayType = HTMLMediaElement.prototype.canPlayType;
                HTMLMediaElement.prototype.canPlayType = function(type) {{
                    if (type.includes('avc1')) return 'probably';
                    if (type.includes('mp4a.40')) return 'probably';
                    if (type === 'video/mp4') return 'probably';
                    return canPlayType.apply(this, arguments);
                }};

                // 6. WebDriver
                delete Object.getPrototypeOf(navigator).webdriver;

                // 7. Chrome Object
                window.chrome = {{ runtime: {{}} }};
            }})();
        "#,
            ua = self.user_agent(),
            platform = self.os.platform(),
            cores = self.cpu_cores,
            memory = self.memory_gb,
            webgl_vendor = self.gpu.vendor(),
            webgl_renderer = self.gpu.renderer(),
            chrome_ver = self.chrome_version,
            hints_platform = self.os.hints_platform(),
        )
    }
}

impl fmt::Display for ChaserProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ChaserProfile({:?}, Chrome {}, {:?})",
            self.os, self.chrome_version, self.gpu
        )
    }
}

/// Builder for constructing `ChaserProfile` instances
#[derive(Debug, Clone)]
pub struct ChaserProfileBuilder {
    os: Os,
    chrome_version: u32,
    gpu: Gpu,
    memory_gb: u32,
    cpu_cores: u32,
    locale: String,
    timezone: String,
    screen_width: u32,
    screen_height: u32,
}

impl ChaserProfileBuilder {
    /// Set the Chrome version (default: 129)
    pub fn chrome_version(mut self, version: u32) -> Self {
        self.chrome_version = version;
        self
    }

    /// Set the GPU for WebGL spoofing
    pub fn gpu(mut self, gpu: Gpu) -> Self {
        self.gpu = gpu;
        self
    }

    /// Set device memory in GB (default: 8)
    pub fn memory_gb(mut self, gb: u32) -> Self {
        self.memory_gb = gb;
        self
    }

    /// Set CPU core count (default: 8)
    pub fn cpu_cores(mut self, cores: u32) -> Self {
        self.cpu_cores = cores;
        self
    }

    /// Set the locale (e.g., "en-US", "de-DE")
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Set the timezone (e.g., "America/New_York", "Europe/Berlin")
    pub fn timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = tz.into();
        self
    }

    /// Set screen resolution
    pub fn screen(mut self, width: u32, height: u32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    /// Build the final profile
    pub fn build(self) -> ChaserProfile {
        ChaserProfile {
            os: self.os,
            chrome_version: self.chrome_version,
            gpu: self.gpu,
            memory_gb: self.memory_gb,
            cpu_cores: self.cpu_cores,
            locale: self.locale,
            timezone: self.timezone,
            screen_width: self.screen_width,
            screen_height: self.screen_height,
        }
    }
}

// Re-export the old trait-based system for backwards compatibility
pub use crate::stealth::{LinuxProfile, MacOSProfile, StealthProfile, WindowsNvidiaProfile};
