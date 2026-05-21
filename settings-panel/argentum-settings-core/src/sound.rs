//! PipeWire sound backend via `pactl` (PipeWire ships a PulseAudio-compatible
//! socket so PulseAudio CLI tools work transparently). We model only the
//! output side: enumerate sinks, set the default, set its volume.

use crate::{Error, Result, on_runtime};

#[derive(Debug, Clone)]
pub struct AudioSink {
    /// pactl's stable string name, e.g. `alsa_output.pci-0000_00_1f.3.analog-stereo`.
    pub name: String,
    /// Human-readable description from `pactl list sinks` (e.g. "Built-in Audio Analog Stereo").
    pub description: String,
    pub is_default: bool,
    /// 0-100, averaged across channels. None if unknown.
    pub volume_percent: Option<u32>,
}

pub async fn list_sinks() -> Result<Vec<AudioSink>> {
    on_runtime(async {
        // `pactl list short sinks` gives one tab-separated line per sink:
        //   id  name  driver  format  state
        let short = tokio::process::Command::new("pactl")
            .args(["list", "short", "sinks"])
            .output()
            .await?;
        if !short.status.success() {
            return Ok(Vec::new());
        }
        let names = parse_short_sinks(&String::from_utf8_lossy(&short.stdout));

        // Resolve the default sink name (one line).
        let default_out = tokio::process::Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .await?;
        let default_name = String::from_utf8_lossy(&default_out.stdout).trim().to_string();

        // Pull rich descriptions + volume from `pactl list sinks`.
        let full = tokio::process::Command::new("pactl")
            .args(["list", "sinks"])
            .output()
            .await?;
        let details = parse_full_sinks(&String::from_utf8_lossy(&full.stdout));

        let mut out = Vec::new();
        for name in names {
            let detail = details.iter().find(|d| d.0 == name);
            let description = detail.map(|d| d.1.clone()).unwrap_or_else(|| name.clone());
            let volume_percent = detail.and_then(|d| d.2);
            out.push(AudioSink {
                name: name.clone(),
                description,
                is_default: name == default_name,
                volume_percent,
            });
        }
        Ok(out)
    })
    .await
}

pub async fn set_default_sink(name: &str) -> Result<()> {
    let name = name.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("pactl")
            .args(["set-default-sink", &name])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("pactl set-default-sink {name}"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

pub async fn set_sink_volume(name: &str, percent: u32) -> Result<()> {
    let name = name.to_string();
    on_runtime(async move {
        let out = tokio::process::Command::new("pactl")
            .args(["set-sink-volume", &name, &format!("{percent}%")])
            .output()
            .await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Subprocess {
                cmd: format!("pactl set-sink-volume {name} {percent}%"),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    })
    .await
}

pub fn parse_short_sinks(input: &str) -> Vec<String> {
    input
        .lines()
        .filter_map(|line| line.split('\t').nth(1).map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Returns `[(name, description, volume_percent)]`. Parses `pactl list sinks`
/// (the long form) by walking `Sink #N` blocks.
pub fn parse_full_sinks(input: &str) -> Vec<(String, String, Option<u32>)> {
    let mut out: Vec<(String, String, Option<u32>)> = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_desc: Option<String> = None;
    let mut cur_vol: Option<u32> = None;
    let flush = |out: &mut Vec<(String, String, Option<u32>)>,
                 name: &mut Option<String>,
                 desc: &mut Option<String>,
                 vol: &mut Option<u32>| {
        if let Some(n) = name.take() {
            out.push((n, desc.take().unwrap_or_default(), vol.take()));
        }
    };
    for raw in input.lines() {
        let line = raw.trim_start();
        if line.starts_with("Sink #") {
            flush(&mut out, &mut cur_name, &mut cur_desc, &mut cur_vol);
        } else if let Some(rest) = line.strip_prefix("Name: ") {
            cur_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Description: ") {
            cur_desc = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Volume: ") {
            cur_vol = extract_volume_percent(rest);
        }
    }
    flush(&mut out, &mut cur_name, &mut cur_desc, &mut cur_vol);
    out
}

/// "front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB"
/// → 50. We average across all channels found.
fn extract_volume_percent(line: &str) -> Option<u32> {
    let mut sum: u64 = 0;
    let mut n: u64 = 0;
    for token in line.split('/') {
        let token = token.trim();
        if let Some(num) = token.strip_suffix('%') {
            if let Ok(v) = num.trim().parse::<u32>() {
                sum += v as u64;
                n += 1;
            }
        }
    }
    if n == 0 { None } else { Some((sum / n) as u32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_sinks() {
        let raw = "44\talsa_output.foo\tPipeWire\ts32le 2ch 48000Hz\tSUSPENDED\n45\talsa_output.bar\tPipeWire\ts32le 2ch 48000Hz\tRUNNING\n";
        let r = parse_short_sinks(raw);
        assert_eq!(r, vec!["alsa_output.foo", "alsa_output.bar"]);
    }

    #[test]
    fn parses_long_sinks_volume() {
        let raw = r#"
Sink #44
    State: SUSPENDED
    Name: alsa_output.foo
    Description: Built-in Audio Analog Stereo
    Volume: front-left: 32768 /  50% / -18.06 dB,   front-right: 32768 /  50% / -18.06 dB
Sink #45
    Name: alsa_output.bar
    Description: HDMI Audio
    Volume: front-left: 65536 / 100% / 0.00 dB
"#;
        let r = parse_full_sinks(raw);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].0, "alsa_output.foo");
        assert_eq!(r[0].1, "Built-in Audio Analog Stereo");
        assert_eq!(r[0].2, Some(50));
        assert_eq!(r[1].2, Some(100));
    }
}
