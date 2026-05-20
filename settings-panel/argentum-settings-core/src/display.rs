//! Display configuration via `xrandr` (X11). Wayland is stubbed.

use crate::{Error, Result, on_runtime};

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub current: Option<Mode>,
    pub scale: f32,
    pub modes: Vec<Mode>,
    pub connected: bool,
    pub primary: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: f32,
}

impl Mode {
    pub fn label(&self) -> String {
        format!("{}×{} @ {:.0} Hz", self.width, self.height, self.refresh_hz)
    }
}

#[derive(Debug, Clone)]
pub struct MonitorChange {
    pub name: String,
    pub mode: Mode,
    pub scale: f32,
}

pub async fn query() -> Result<Vec<Monitor>> {
    on_runtime(async {
        let out = tokio::process::Command::new("xrandr").arg("--query").output().await;
        let out = match out {
            Ok(o) if o.status.success() => o,
            Ok(o) => {
                return Err(Error::Subprocess {
                    cmd: "xrandr --query".into(),
                    code: o.status.code().unwrap_or(-1),
                    stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
                });
            }
            Err(_) => return Ok(Vec::new()),
        };
        let stdout = String::from_utf8_lossy(&out.stdout);
        Ok(parse_xrandr(&stdout))
    })
    .await
}

pub async fn apply(changes: &[MonitorChange]) -> Result<()> {
    let changes = changes.to_vec();
    on_runtime(async move {
        for c in &changes {
            let mode_arg = format!("{}x{}", c.mode.width, c.mode.height);
            let rate_arg = format!("{:.2}", c.mode.refresh_hz);
            let scale_arg = format!("{:.2}x{:.2}", c.scale, c.scale);
            let out = tokio::process::Command::new("xrandr")
                .args([
                    "--output", &c.name, "--mode", &mode_arg, "--rate", &rate_arg, "--scale", &scale_arg,
                ])
                .output()
                .await?;
            if !out.status.success() {
                return Err(Error::Subprocess {
                    cmd: format!("xrandr --output {} --mode {} --rate {} --scale {}",
                        c.name, mode_arg, rate_arg, scale_arg),
                    code: out.status.code().unwrap_or(-1),
                    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                });
            }
        }
        Ok(())
    })
    .await
}

// TODO: Wayland backend (wlr-randr / org.gnome.Mutter.DisplayConfig).
#[allow(dead_code)]
async fn wayland_query() -> Result<Vec<Monitor>> {
    Err(Error::NotImplemented("Wayland display backend (wlr-randr)"))
}

pub fn parse_xrandr(input: &str) -> Vec<Monitor> {
    let mut monitors: Vec<Monitor> = Vec::new();
    let mut current: Option<Monitor> = None;

    for line in input.lines() {
        if line.starts_with("Screen ") {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(prev) = current.take() {
                monitors.push(prev);
            }
            let mut parts = line.split_whitespace();
            let name = match parts.next() {
                Some(n) => n.to_string(),
                None => continue,
            };
            let status = parts.next().unwrap_or("");
            let connected = status == "connected";
            let primary = line.contains(" primary ");
            current = Some(Monitor { name, current: None, scale: 1.0, modes: Vec::new(), connected, primary });
            continue;
        }
        let Some(mon) = current.as_mut() else { continue };
        let mut tokens = line.split_whitespace();
        let Some(res) = tokens.next() else { continue };
        let Some((w, h)) = res.split_once('x') else { continue };
        let (Ok(width), Ok(height)) = (w.parse::<u32>(), h.parse::<u32>()) else { continue };
        for rate_token in tokens {
            let is_current = rate_token.contains('*');
            let cleaned: String = rate_token.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            let Ok(refresh_hz) = cleaned.parse::<f32>() else { continue };
            let mode = Mode { width, height, refresh_hz };
            if is_current {
                mon.current = Some(mode.clone());
            }
            mon.modes.push(mode);
        }
    }
    if let Some(prev) = current.take() {
        monitors.push(prev);
    }
    monitors
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
Screen 0: minimum 320 x 200, current 1920 x 1080, maximum 8192 x 8192
HDMI-1 connected primary 1920x1080+0+0 (normal left inverted right x axis y axis) 510mm x 287mm
   1920x1080     60.00*+  59.93    50.00
   1680x1050     59.95
DP-2 disconnected (normal left inverted right x axis y axis)
";

    #[test]
    fn parses_primary_with_modes() {
        let m = parse_xrandr(SAMPLE);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].name, "HDMI-1");
        assert!(m[0].primary);
        assert!(m[0].connected);
        let cur = m[0].current.as_ref().unwrap();
        assert_eq!((cur.width, cur.height), (1920, 1080));
        assert!((cur.refresh_hz - 60.0).abs() < 0.1);
        assert_eq!(m[1].name, "DP-2");
        assert!(!m[1].connected);
    }
}
