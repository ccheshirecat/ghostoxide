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
    device_pixel_ratio: f32,
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
        // OS-specific defaults for consistency
        let (screen_width, screen_height, device_pixel_ratio, cpu_cores) = match os {
            Os::Windows => (1920, 1080, 1.0, 8),
            Os::MacOSIntel => (1440, 900, 2.0, 8),
            Os::MacOSArm => (1728, 1117, 2.0, 14), // M4 Max defaults
            Os::Linux => (1920, 1080, 1.0, 8),
        };

        ChaserProfileBuilder {
            os,
            chrome_version: 131, // Keep reasonably current
            gpu: match os {
                Os::Windows => Gpu::NvidiaRTX3080,
                Os::MacOSIntel => Gpu::AppleM1Pro,
                Os::MacOSArm => Gpu::AppleM4Max,
                Os::Linux => Gpu::NvidiaGTX1660,
            },
            memory_gb: 8,
            cpu_cores,
            locale: "en-US".to_string(),
            timezone: "America/New_York".to_string(),
            screen_width,
            screen_height,
            device_pixel_ratio,
        }
    }

    /// Create a Windows profile with sensible defaults (RTX 3080, 8 cores)
    pub fn windows() -> ChaserProfileBuilder {
        Self::new(Os::Windows)
    }

    /// Create a macOS Intel profile (realistic MacBook Pro defaults)
    pub fn macos_intel() -> ChaserProfileBuilder {
        Self::new(Os::MacOSIntel)
    }

    /// Create a macOS Apple Silicon profile (M4 Max defaults from real device)
    pub fn macos_arm() -> ChaserProfileBuilder {
        Self::new(Os::MacOSArm)
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
    pub fn device_pixel_ratio(&self) -> f32 {
        self.device_pixel_ratio
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
    /// Uses makeNative helper and prototype-based properties for full stealth
    pub fn bootstrap_script(&self) -> String {
        format!(
            r#"
            (function() {{
                // === chaser-oxide "GOD MODE" STEALTH ===
                // Profile: {ua}

                // Helper: Make functions appear native (recursive toString protection)
                const makeNative = (func, name) => {{
                    // 1. Rename the function
                    Object.defineProperty(func, 'name', {{ value: name }});
                    
                    // 2. Create the "native" string
                    const nativeStr = `function ${{name}}() {{ [native code] }}`;
                    
                    // 3. Create the toString function
                    const newToString = function() {{ return nativeStr; }};
                    
                    // 4. Spoof toString of toString (Recursive Stealth - prevents toString.toString() leak)
                    Object.defineProperty(newToString, 'toString', {{
                        value: function() {{ return "function toString() {{ [native code] }}"; }}
                    }});
                    Object.defineProperty(newToString, 'name', {{ value: 'toString' }});
                    
                    // 5. Apply it
                    Object.defineProperty(func, 'toString', {{
                        value: newToString,
                        writable: true, enumerable: false, configurable: true
                    }});
                    
                    return func;
                }};

                // Get Navigator prototype (properties should live here for hasOwnProperty check)
                const navProto = Object.getPrototypeOf(navigator);

                // 1. Platform (on prototype)
                Object.defineProperty(navProto, 'platform', {{
                    get: makeNative(function() {{ return '{platform}'; }}, 'get platform'),
                    configurable: true, enumerable: true
                }});

                // 2. Hardware (on prototype)
                Object.defineProperty(navProto, 'hardwareConcurrency', {{
                    get: makeNative(function() {{ return {cores}; }}, 'get hardwareConcurrency'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'deviceMemory', {{
                    get: makeNative(function() {{ return {memory}; }}, 'get deviceMemory'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(navProto, 'maxTouchPoints', {{
                    get: makeNative(function() {{ return 0; }}, 'get maxTouchPoints'),
                    configurable: true, enumerable: true
                }});

                // 2b. Screen & DPR
                Object.defineProperty(window, 'devicePixelRatio', {{
                    get: makeNative(function() {{ return {dpr}; }}, 'get devicePixelRatio'),
                    configurable: true, enumerable: true
                }});
                Object.defineProperty(screen, 'width', {{
                    get: makeNative(function() {{ return {screen_w}; }}, 'get width'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'height', {{
                    get: makeNative(function() {{ return {screen_h}; }}, 'get height'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'availWidth', {{
                    get: makeNative(function() {{ return {screen_w}; }}, 'get availWidth'),
                    configurable: true
                }});
                Object.defineProperty(screen, 'availHeight', {{
                    get: makeNative(function() {{ return {screen_h}; }}, 'get availHeight'),
                    configurable: true
                }});

                // 3. WebGL (with native toString and error.stack protection)
                const spoofWebGL = (proto) => {{
                    const originalGetParameter = proto.getParameter;
                    proto.getParameter = makeNative(function(parameter) {{
                        try {{
                            if (parameter === 37445) return '{webgl_vendor}';
                            if (parameter === 37446) return '{webgl_renderer}';
                            return originalGetParameter.apply(this, arguments);
                        }} catch(e) {{
                            // Scrub stack trace to hide proxy
                            if (e && e.stack) {{
                                e.stack = e.stack.split('\\n').filter(line => 
                                    !line.includes('Object.apply') && !line.includes('<anonymous>')
                                ).join('\\n');
                            }}
                            throw e;
                        }}
                    }}, 'getParameter');
                }};
                try {{
                    spoofWebGL(WebGLRenderingContext.prototype);
                    if (typeof WebGL2RenderingContext !== 'undefined') {{
                        spoofWebGL(WebGL2RenderingContext.prototype);
                    }}
                }} catch(e) {{}}

                // 4. Client Hints (with getHighEntropyValues)
                Object.defineProperty(navProto, 'userAgentData', {{
                    get: makeNative(function() {{
                        return {{
                            brands: [
                                {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                {{ brand: "Chromium", version: "{chrome_ver}" }},
                                {{ brand: "Not=A?Brand", version: "24" }}
                            ],
                            mobile: false,
                            platform: "{hints_platform}",
                            getHighEntropyValues: makeNative(async function(hints) {{
                                return {{
                                    architecture: "x86",
                                    bitness: "64",
                                    brands: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}" }},
                                        {{ brand: "Not=A?Brand", version: "24" }}
                                    ],
                                    fullVersionList: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}.0.0.0" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}.0.0.0" }},
                                        {{ brand: "Not=A?Brand", version: "24.0.0.0" }}
                                    ],
                                    mobile: false,
                                    model: "",
                                    platform: "{hints_platform}",
                                    platformVersion: "10.0.0",
                                    uaFullVersion: "{chrome_ver}.0.0.0"
                                }};
                            }}, 'getHighEntropyValues'),
                            toJSON: makeNative(function() {{
                                return {{
                                    brands: [
                                        {{ brand: "Google Chrome", version: "{chrome_ver}" }},
                                        {{ brand: "Chromium", version: "{chrome_ver}" }},
                                        {{ brand: "Not=A?Brand", version: "24" }}
                                    ],
                                    mobile: false,
                                    platform: "{hints_platform}"
                                }};
                            }}, 'toJSON')
                        }};
                    }}, 'get userAgentData'),
                    configurable: true, enumerable: true
                }});

                // 5. Video Codecs (with native toString)
                const originalCanPlayType = HTMLMediaElement.prototype.canPlayType;
                HTMLMediaElement.prototype.canPlayType = makeNative(function(type) {{
                    if (!type) return originalCanPlayType.apply(this, arguments);
                    if (type.includes('avc1') || type.includes('mp4a.40') || type === 'video/mp4' || type === 'audio/mp4') {{
                        return 'probably';
                    }}
                    return originalCanPlayType.apply(this, arguments);
                }}, 'canPlayType');

                // 6. WebDriver (delete and override with undefined getter)
                try {{ delete Object.getPrototypeOf(navigator).webdriver; }} catch(e) {{}}
                Object.defineProperty(navProto, 'webdriver', {{
                    get: makeNative(function() {{ return undefined; }}, 'get webdriver'),
                    configurable: true, enumerable: true
                }});

                // 7. Chrome Object (basic structure - expanded in chrome_runtime_mock)
                if (!window.chrome) {{
                    window.chrome = {{ runtime: {{}} }};
                }}
            }})();
        "#,
            ua = self.user_agent(),
            platform = self.os.platform(),
            cores = self.cpu_cores,
            memory = self.memory_gb,
            dpr = self.device_pixel_ratio,
            screen_w = self.screen_width,
            screen_h = self.screen_height,
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
    device_pixel_ratio: f32,
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

    /// Set device pixel ratio (1.0 for standard, 2.0 for Retina/HiDPI)
    pub fn device_pixel_ratio(mut self, dpr: f32) -> Self {
        self.device_pixel_ratio = dpr;
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
            device_pixel_ratio: self.device_pixel_ratio,
        }
    }
}
