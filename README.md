# Ghostoxide

**A Rust-based fork of `chromiumoxide` modified for hardened browser automation.**

Ghostoxide is an experimental fork of the `chromiumoxide` library. It incorporates modifications to the core Chrome DevTools Protocol (CDP) client and high-level interaction utilities to reduce the footprint of automated browser sessions.

## Core Modifications

### 1. Protocol-Level Stealth

Standard CDP clients trigger internal browser signals during initialization. Ghostoxide modifies these behaviors:

* **`Runtime.enable` Mitigation**: Standard automation often relies on `Runtime.enable`, which exposes a distinct fingerprint. Ghostoxide utilizes `Page.createIsolatedWorld` to execute scripts in a secondary environment that is not subject to the same detection vectors as the primary world.
* **Utility World Renaming**: The default "Puppeteer" or "Chromiumoxide" utility world names have been renamed to neutralize simple string-based environment checks.

### 2. Fingerprint Synchronization (Hardware Harmony)

Anti-bot systems look for discrepancies between the reported User-Agent and the browser's execution environment.

* **State Management**: Ghostoxide injects scripts during document creation to synchronize `navigator.platform`, `WebGL` vendor/renderer strings, and hardware concurrency values with the configured profile.
* **Consistency Enforcement**: These values are enforced via the `IsolatedWorld` mechanism to ensure they are available before the target site’s scripts execute.

### 3. Human Interaction Simulation

Ghostoxide includes a native physics-based input engine to avoid synthetic interaction patterns.

* **Bezier Mouse Curves**: Mouse movements follow randomized Bezier paths with acceleration and deceleration profiles rather than linear coordinate jumps.
* **Typing Physics**: Keypresses include variable inter-character delays and optional typo-correction simulation to mimic human typing cadences.

## Technical Comparison

| Metric | Ghostoxide | Node.js Alternatives |
| --- | --- | --- |
| **Language** | Rust | JavaScript |
| **Memory Footprint** | ~50MB - 100MB (per process) | ~500MB+ (per process) |
| **Transport Patching** | Protocol-level (Internal Fork) | High-level (Wrapper/Plugin) |

## Usage

```rust
use ghostoxide::{Browser, BrowserConfig, GhostPage, GhostProfile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize with a consistent Windows profile
    let profile = GhostProfile::windows().build();
    
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_profile(profile)
            .build()?
    ).await?;

    // 2. Wrap the standard Page in a GhostPage for interaction utilities
    let page = browser.new_page("https://example.com").await?;
    let ghost = GhostPage::new(page);

    // 3. Use hardened interaction methods
    ghost.move_mouse_human(400, 300).await?;
    ghost.type_text("Search query").await?;

    Ok(())
}

```

## Acknowledgements

This project is a specialized fork of **[chromiumoxide](https://github.com/mattsse/chromiumoxide)**. The core CDP client and session management are derived from their excellent work.

## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](https://www.google.com/search?q=LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](https://www.google.com/search?q=LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))