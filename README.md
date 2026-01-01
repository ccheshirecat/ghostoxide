# chaser-oxide

**A Rust-based fork of `chromiumoxide` modified for hardened browser automation.**

chaser-oxide is an experimental fork of the `chromiumoxide` library. It incorporates modifications to the core Chrome DevTools Protocol (CDP) client and high-level interaction utilities to reduce the footprint of automated browser sessions.

## Features

- **Protocol-Level Stealth**: Patches CDP at the transport layer, not via JavaScript wrappers
- **Fingerprint Profiles**: Pre-configured Windows, Linux, macOS profiles with consistent hardware fingerprints
- **Human Interaction Engine**: Physics-based mouse movements and realistic typing patterns
- **Request Interception**: Built-in request modification and blocking capabilities
- **Low Memory Footprint**: ~50-100MB vs ~500MB+ for Node.js alternatives

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
chaser-oxide = { git = "https://github.com/ccheshirecat/chaser-oxide" }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## Requirements

- Rust 1.75+
- Chrome/Chromium browser installed
- Supported platforms: Windows, macOS, Linux

## Quick Start

```rust
use chaser_oxide::{Browser, BrowserConfig, ChaserPage, ChaserProfile};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create a fingerprint profile
    let profile = ChaserProfile::windows().build();
    
    // 2. Launch browser
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder().build()?
    ).await?;

    tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });

    // 3. Create page and wrap in ChaserPage
    let page = browser.new_page("about:blank").await?;
    let chaser = ChaserPage::new(page);

    // 4. Apply profile (sets UA + injects stealth scripts) - BEFORE navigation
    chaser.apply_profile(&profile).await?;

    // 5. Navigate to target
    chaser.inner().goto("https://example.com").await?;

    // 6. Use human-like interaction methods
    chaser.move_mouse_human(400.0, 300.0).await?;
    chaser.click_human(500.0, 400.0).await?;
    chaser.type_text("Search query").await?;

    Ok(())
}
```

## API Reference

### ChaserProfile Builder

Create customized browser fingerprint profiles:

```rust
use chaser_oxide::{ChaserProfile, Gpu};

// Quick presets
let windows = ChaserProfile::windows().build();
let linux = ChaserProfile::linux().build();
let mac_arm = ChaserProfile::macos_arm().build();
let mac_intel = ChaserProfile::macos_intel().build();

// Custom profile with builder
let custom = ChaserProfile::windows()
    .chrome_version(130)           // Chrome version for UA
    .gpu(Gpu::NvidiaRTX4080)       // WebGL renderer
    .memory_gb(32)                 // navigator.deviceMemory
    .cpu_cores(16)                 // navigator.hardwareConcurrency
    .locale("de-DE")               // navigator.language
    .timezone("Europe/Berlin")     // Intl timezone
    .screen_size(2560, 1440)       // screen.width/height
    .build();
```

### Available GPUs

```rust
pub enum Gpu {
    // NVIDIA
    NvidiaRTX4090, NvidiaRTX4080, NvidiaRTX4070,
    NvidiaRTX3090, NvidiaRTX3080, NvidiaRTX3070, NvidiaRTX3060,
    NvidiaGTX1660, NvidiaGTX1080,
    // AMD
    AmdRX7900XTX, AmdRX6800XT, AmdRX6700XT,
    // Intel
    IntelUHD630, IntelIrisXe,
    // Apple
    AppleM1, AppleM1Pro, AppleM2, AppleM3, AppleM4Max,
}
```

### ChaserPage Methods

```rust
impl ChaserPage {
    // Profile & Stealth
    async fn apply_profile(&self, profile: &ChaserProfile) -> Result<()>;
    
    // Human-like Mouse Movement (Bezier curves)
    async fn move_mouse_human(&self, x: f64, y: f64) -> Result<()>;
    async fn click_human(&self, x: f64, y: f64) -> Result<()>;
    async fn scroll_human(&self, delta_y: i32) -> Result<()>;
    
    // Human-like Typing
    async fn type_text(&self, text: &str) -> Result<()>;
    async fn type_text_with_typos(&self, text: &str) -> Result<()>;
    
    // Stealth Script Execution
    async fn evaluate_stealth<T>(&self, expression: &str) -> Result<T>;
    
    // Request Interception
    async fn enable_request_interception(&self, pattern: &str, resource_type: Option<ResourceType>) -> Result<()>;
    async fn disable_request_interception(&self) -> Result<()>;
    async fn fulfill_request_html(&self, request_id: RequestId, html: &str, status: u16) -> Result<()>;
    async fn continue_request(&self, request_id: RequestId) -> Result<()>;
    async fn block_request(&self, request_id: RequestId) -> Result<()>;
    
    // Access underlying Page
    fn inner(&self) -> &Page;
}
```

### BrowserConfig

```rust
let config = BrowserConfig::builder()
    .chrome_executable("/path/to/chrome")  // Custom Chrome path
    .with_head()                           // Show browser window (default)
    .headless()                            // Run headless
    .viewport(Viewport {
        width: 1920,
        height: 1080,
        device_scale_factor: None,
        emulating_mobile: false,
        is_landscape: false,
        has_touch: false,
    })
    .build()?;
```

## Core Modifications

### 1. Protocol-Level Stealth

Standard CDP clients trigger internal browser signals during initialization. chaser-oxide modifies these behaviors:

* **`Runtime.enable` Mitigation**: Uses `Page.createIsolatedWorld` to execute scripts in a secondary environment that bypasses detection vectors.
* **Utility World Renaming**: The default "Puppeteer" or "Chromiumoxide" utility world names have been neutralized.

### 2. Fingerprint Synchronization

Anti-bot systems look for discrepancies between the reported User-Agent and the browser's execution environment.

* **State Management**: Injects scripts during document creation to synchronize `navigator.platform`, `WebGL` vendor/renderer strings, and hardware concurrency values.
* **Consistency Enforcement**: Values are enforced via the `IsolatedWorld` mechanism to ensure they are available before the target site's scripts execute.

### 3. Human Interaction Simulation

* **Bezier Mouse Curves**: Mouse movements follow randomized Bezier paths with acceleration and deceleration profiles.
* **Typing Physics**: Keypresses include variable inter-character delays and optional typo-correction simulation.

## Technical Comparison

| Metric | chaser-oxide | Node.js Alternatives |
| --- | --- | --- |
| **Language** | Rust | JavaScript |
| **Memory Footprint** | ~50MB - 100MB (per process) | ~500MB+ (per process) |
| **Transport Patching** | Protocol-level (Internal Fork) | High-level (Wrapper/Plugin) |

## Dependencies

- [chromiumoxide](https://github.com/mattsse/chromiumoxide) - Base CDP client (forked)
- [tokio](https://tokio.rs) - Async runtime
- [futures](https://docs.rs/futures) - Async utilities

## Acknowledgements

This project is a specialized fork of **[chromiumoxide](https://github.com/mattsse/chromiumoxide)**. The core CDP client and session management are derived from their excellent work.

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
