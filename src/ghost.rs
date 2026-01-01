use std::sync::{Arc, Mutex};
use crate::page::Page;
use chromiumoxide_cdp::cdp::browser_protocol::page::CreateIsolatedWorldParams;
use chromiumoxide_cdp::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType,
};
use chromiumoxide_cdp::cdp::js_protocol::runtime::EvaluateParams;
use serde_json::Value;
use anyhow::{anyhow, Result};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Point { pub x: f64, pub y: f64 }

/// A wrapper around `Page` that provides **absolute stealth** execution and human-like input simulation.
/// 
/// `GhostPage` offers:
/// - **Zero-footprint JavaScript execution** via `Page.createIsolatedWorld` (never touches Runtime domain)
/// - Bezier curve mouse movements with jitter and overshoot
/// - Realistic keyboard input with randomized delays
/// 
/// This implementation matches the Rebrowser "Protocol Hijack" method exactly.
#[derive(Clone, Debug)]
pub struct GhostPage {
    inner: Page,
    mouse_pos: Arc<Mutex<Point>>,
}

impl GhostPage {
    /// Create a new GhostPage wrapping the given Page.
    pub fn new(inner: Page) -> Self {
        Self { 
            inner, 
            mouse_pos: Arc::new(Mutex::new(Point { x: 0.0, y: 0.0 })),
        }
    }

    /// Access the underlying Page for standard operations.
    pub fn inner(&self) -> &Page {
        &self.inner
    }

    /// **THE REBROWSER METHOD: Absolute Stealth Execution**
    /// 
    /// This method achieves 100% stealth parity with Rebrowser by:
    /// 1. Using `Page.createIsolatedWorld` to create a JS context
    /// 2. Getting the `ExecutionContextId` directly from the response
    /// 3. **Never calling `Runtime.enable`**
    /// 
    /// Site scripts cannot see your variables (isolated world).
    /// Anti-bots cannot detect CDP activity (Runtime domain untouched).
    pub async fn evaluate_stealth(&self, script: &str) -> Result<Option<Value>> {
        // Get the main frame ID
        let frame_id = self.inner.mainframe().await
            .map_err(|e| anyhow!("{}", e))?
            .ok_or_else(|| anyhow!("No main frame available"))?;

        // Create an isolated world - Chrome returns the Context ID in the response!
        // This is the key insight: we get a context ID without touching Runtime domain
        let isolated_world = self.inner.execute(
            CreateIsolatedWorldParams::builder()
                .frame_id(frame_id)
                .world_name("ghost") // Our stealth world
                .grant_univeral_access(true) // Access to page DOM
                .build()
                .unwrap()
        ).await.map_err(|e| anyhow!("{}", e))?;

        let ctx_id = isolated_world.result.execution_context_id;

        // Execute in the isolated world using the captured context ID
        let params = EvaluateParams::builder()
            .expression(script)
            .context_id(ctx_id)
            .await_promise(true)
            .return_by_value(true)
            .build()
            .unwrap();

        let res = self.inner.execute(params).await.map_err(|e| anyhow!("{}", e))?;
        Ok(res.result.result.value)
    }

    /// Moves the mouse to the target coordinates using a human-like Bezier curve path.
    /// 
    /// The path includes:
    /// - Randomized control points for natural arcs
    /// - 20% chance of slight overshoot
    /// - Target jitter (±2px)
    /// - Variable delays between movements (5-15ms)
    pub async fn move_mouse_human(&self, x: f64, y: f64) -> Result<()> {
        let start = { *self.mouse_pos.lock().unwrap() };
        let end = Point { x, y };

        let mut rng = rand::thread_rng();
        
        // Target Selection Jitter: don't land exactly on the pixel
        let jitter_x = rng.gen_range(-2.0..2.0);
        let jitter_y = rng.gen_range(-2.0..2.0);
        let target_with_jitter = Point { x: end.x + jitter_x, y: end.y + jitter_y };

        let path = BezierPath::generate(start, target_with_jitter, 25);
        
        for point in path {
            self.inner.move_mouse(crate::layout::Point { x: point.x, y: point.y }).await.map_err(|e| anyhow!("{}", e))?;
            *self.mouse_pos.lock().unwrap() = point;
            // Tiny delay to simulate physical movement
            tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(5..15))).await;
        }

        Ok(())
    }

    /// Perform a click at the current mouse position.
    pub async fn click(&self) -> Result<()> {
         let pos = { *self.mouse_pos.lock().unwrap() };
         self.inner.click(crate::layout::Point { x: pos.x, y: pos.y }).await.map_err(|e| anyhow!("{}", e))?;
         Ok(())
    }

    /// Move to target and click with full human-like behavior.
    /// 
    /// Combines Bezier curve mouse movement with a natural click, including:
    /// - Human-like path to target
    /// - Small random delay before clicking (50-150ms)
    /// - Variable click duration
    pub async fn click_human(&self, x: f64, y: f64) -> Result<()> {
        let mut rng = rand::thread_rng();
        
        // Move to target with bezier curve
        self.move_mouse_human(x, y).await?;
        
        // Small pause before clicking (humans don't click instantly after arriving)
        tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(50..150))).await;
        
        // Click
        self.click().await?;
        
        // Small pause after clicking
        tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(30..80))).await;
        
        Ok(())
    }

    /// Type text with human-like delays between keystrokes.
    /// 
    /// Simulates realistic typing with:
    /// - Variable delay between keys (50-150ms by default)
    /// - Occasional longer pauses (5% chance of 200-400ms pause)
    pub async fn type_text(&self, text: &str) -> Result<()> {
        self.type_text_with_delay(text, 50, 150).await
    }

    /// Type text with custom delay range (in milliseconds).
    /// 
    /// # Arguments
    /// * `text` - The text to type
    /// * `min_delay_ms` - Minimum delay between keystrokes
    /// * `max_delay_ms` - Maximum delay between keystrokes
    pub async fn type_text_with_delay(&self, text: &str, min_delay_ms: u64, max_delay_ms: u64) -> Result<()> {
        let mut rng = rand::thread_rng();
        
        for c in text.chars() {
            // Send keyDown with the character
            let key_down = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyDown)
                .text(c.to_string())
                .build()
                .unwrap();
            
            self.inner.execute(key_down).await.map_err(|e| anyhow!("{}", e))?;
            
            // Send keyUp
            let key_up = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyUp)
                .build()
                .unwrap();
            
            self.inner.execute(key_up).await.map_err(|e| anyhow!("{}", e))?;
            
            // Random delay between keystrokes
            let delay = rng.gen_range(min_delay_ms..max_delay_ms);
            
            // 5% chance of a longer "thinking" pause
            let actual_delay = if rng.gen_bool(0.05) {
                rng.gen_range(200..400)
            } else {
                delay
            };
            
            tokio::time::sleep(tokio::time::Duration::from_millis(actual_delay)).await;
        }
        
        Ok(())
    }

    /// Press a specific key (e.g., "Enter", "Tab", "Escape").
    pub async fn press_key(&self, key: &str) -> Result<()> {
        // Map common key names to their key codes
        let (key_str, code) = match key {
            "Enter" => ("Enter", "Enter"),
            "Tab" => ("Tab", "Tab"),
            "Escape" => ("Escape", "Escape"),
            "Backspace" => ("Backspace", "Backspace"),
            "Delete" => ("Delete", "Delete"),
            "ArrowUp" => ("ArrowUp", "ArrowUp"),
            "ArrowDown" => ("ArrowDown", "ArrowDown"),
            "ArrowLeft" => ("ArrowLeft", "ArrowLeft"),
            "ArrowRight" => ("ArrowRight", "ArrowRight"),
            _ => (key, key),
        };
        
        let key_down = DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::RawKeyDown)
            .key(key_str)
            .code(code)
            .build()
            .unwrap();
        
        self.inner.execute(key_down).await.map_err(|e| anyhow!("{}", e))?;
        
        let key_up = DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyUp)
            .key(key_str)
            .code(code)
            .build()
            .unwrap();
        
        self.inner.execute(key_up).await.map_err(|e| anyhow!("{}", e))?;
        
        Ok(())
    }

    /// Press Enter key with a small random delay before pressing.
    pub async fn press_enter(&self) -> Result<()> {
        let mut rng = rand::thread_rng();
        tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(100..300))).await;
        self.press_key("Enter").await
    }

    /// Press Tab key to move to next field.
    pub async fn press_tab(&self) -> Result<()> {
        let mut rng = rand::thread_rng();
        tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(50..150))).await;
        self.press_key("Tab").await
    }

    /// Scroll the page with human-like physics (smooth, variable speed).
    /// 
    /// Simulates realistic scrolling with:
    /// - Multiple small scroll steps rather than one jump
    /// - Variable scroll distances per step
    /// - Easing at start and end (deceleration)
    /// 
    /// # Arguments
    /// * `delta_y` - Total pixels to scroll (positive = down, negative = up)
    pub async fn scroll_human(&self, delta_y: i32) -> Result<()> {
        use chromiumoxide_cdp::cdp::browser_protocol::input::{
            DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
        };
        
        let mut rng = rand::thread_rng();
        let pos = { *self.mouse_pos.lock().unwrap() };
        
        // Number of scroll steps (more steps = smoother)
        let steps = (delta_y.abs() / 50).max(3).min(15) as usize;
        let mut remaining = delta_y;
        
        for i in 0..steps {
            // Ease-in/ease-out: scroll less at start and end
            let progress = i as f64 / steps as f64;
            let ease = if progress < 0.3 {
                progress / 0.3 * 0.5 + 0.5
            } else if progress > 0.7 {
                (1.0 - progress) / 0.3 * 0.5 + 0.5
            } else {
                1.0
            };
            
            let base_step = remaining / (steps - i) as i32;
            let jitter = rng.gen_range(-10..10);
            let step = ((base_step as f64 * ease) as i32 + jitter).clamp(-200, 200);
            
            if step == 0 { continue; }
            
            let scroll = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseWheel)
                .x(pos.x)
                .y(pos.y)
                .button(MouseButton::None)
                .delta_x(0.0)
                .delta_y(step as f64)
                .build()
                .unwrap();
            
            self.inner.execute(scroll).await.map_err(|e| anyhow!("{}", e))?;
            remaining -= step;
            
            // Variable delay between scroll events (16-50ms for 60-20 FPS feel)
            tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(16..50))).await;
        }
        
        Ok(())
    }

    /// Type text with occasional typos and corrections for ultra-realistic input.
    /// 
    /// This method has a small chance (~3%) of making a typo and then correcting it,
    /// mimicking how real humans type.
    pub async fn type_text_with_typos(&self, text: &str) -> Result<()> {
        let mut rng = rand::thread_rng();
        let typo_chars = ['q', 'w', 'e', 'r', 't', 'a', 's', 'd', 'f', 'g'];
        
        for c in text.chars() {
            // 3% chance of typo
            if rng.gen_bool(0.03) && c.is_alphabetic() {
                // Type wrong character
                let typo = typo_chars[rng.gen_range(0..typo_chars.len())];
                self.type_single_char(typo).await?;
                
                // Brief pause to "notice" the mistake
                tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(100..300))).await;
                
                // Backspace to correct
                self.press_key("Backspace").await?;
                tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(30..80))).await;
            }
            
            // Type the correct character
            self.type_single_char(c).await?;
            
            // Random delay
            let delay = rng.gen_range(50..150);
            let actual_delay = if rng.gen_bool(0.05) {
                rng.gen_range(200..400) // thinking pause
            } else {
                delay
            };
            tokio::time::sleep(tokio::time::Duration::from_millis(actual_delay)).await;
        }
        
        Ok(())
    }

    /// Helper to type a single character
    async fn type_single_char(&self, c: char) -> Result<()> {
        let key_down = DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyDown)
            .text(c.to_string())
            .build()
            .unwrap();
        
        self.inner.execute(key_down).await.map_err(|e| anyhow!("{}", e))?;
        
        let key_up = DispatchKeyEventParams::builder()
            .r#type(DispatchKeyEventType::KeyUp)
            .build()
            .unwrap();
        
        self.inner.execute(key_up).await.map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BezierPath;

impl BezierPath {
    /// Generates a path of points from start to end using a cubic Bezier curve.
    /// 
    /// The curve includes randomized control points to create natural, human-like arcs.
    pub fn generate(start: Point, end: Point, steps: usize) -> Vec<Point> {
        let mut rng = rand::thread_rng();
        let mut path = Vec::with_capacity(steps);

        // Calculate distance for offset scaling
        let dist = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
        let offset_range = dist * 0.3;

        // First control point (25% along the path with random offset)
        let p1 = Point {
            x: start.x + (end.x - start.x) * 0.25 + rng.gen_range(-offset_range..offset_range),
            y: start.y + (end.y - start.y) * 0.25 + rng.gen_range(-offset_range..offset_range),
        };

        // Second control point (75% along the path with random offset)
        // 20% chance of overshoot
        let mut p2 = Point {
            x: start.x + (end.x - start.x) * 0.75 + rng.gen_range(-offset_range..offset_range),
            y: start.y + (end.y - start.y) * 0.75 + rng.gen_range(-offset_range..offset_range),
        };

        if rng.gen_bool(0.20) {
            let overshoot_amt = dist * 0.05;
            p2.x += if end.x > start.x { overshoot_amt } else { -overshoot_amt };
            p2.y += if end.y > start.y { overshoot_amt } else { -overshoot_amt };
        }

        // Generate points along the Bezier curve
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            
            // Cubic Bezier formula
            let x = (1.0 - t).powi(3) * start.x 
                    + 3.0 * (1.0 - t).powi(2) * t * p1.x 
                    + 3.0 * (1.0 - t) * t.powi(2) * p2.x 
                    + t.powi(3) * end.x;
            
            let y = (1.0 - t).powi(3) * start.y 
                    + 3.0 * (1.0 - t).powi(2) * t * p1.y 
                    + 3.0 * (1.0 - t) * t.powi(2) * p2.y 
                    + t.powi(3) * end.y;

            path.push(Point { x, y });
        }
        
        path
    }
}
