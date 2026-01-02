use anyhow::Result;
use chaser_oxide::{Browser, BrowserConfig, ChaserPage, ChaserProfile};
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create profile FIRST
    let profile = ChaserProfile::windows().build();

    println!("Launching chaser-oxide Stealth Browser...");
    // Use profile.configure_browser() to automatically set window size and stealth args
    let (browser, mut handler) = Browser::launch(
        profile
            .configure_browser(BrowserConfig::builder())
            .with_head() // Show browser for testing
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    tokio::spawn(async move { while let Some(_) = handler.next().await {} });

    // Create page with about:blank
    println!("Creating page...");
    let page = browser.new_page("about:blank").await?;

    // Wrap in ChaserPage and apply full stealth profile
    // This sets viewport, DPR, UA, and injects all Chrome mocks
    println!("Applying stealth profile...");
    let chaser = ChaserPage::new(page);
    chaser.apply_profile(&profile).await?;

    // Small delay to ensure scripts are registered
    tokio::time::sleep(Duration::from_millis(100)).await;

    // NOW navigate to the detection test
    println!("Navigating to detection test...");
    chaser.goto("https://bot.sannysoft.com").await?;

    // Wait for page to fully load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Human-like mouse movement
    println!("Simulating human mouse movement...");
    chaser.move_mouse_human(500.0, 300.0).await?;

    // Test stealth execution
    println!("\nReading values from the PAGE (main world sees spoofed values):");

    // Read what the site's JavaScript sees
    let user_agent = chaser.evaluate_stealth("navigator.userAgent").await?;
    println!("  navigator.userAgent = {:?}", user_agent);

    // Wait and take screenshot
    println!("\nWaiting for page to render...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    chaser
        .raw_page()
        .save_screenshot(
            chaser_oxide::page::ScreenshotParams::builder().build(),
            "stealth_test.png",
        )
        .await?;
    println!("Screenshot saved to stealth_test.png");

    println!("\nBrowser will close in 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
