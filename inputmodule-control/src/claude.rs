//! Show the status of Claude Code sessions on the LED matrix.
//!
//! Claude Code runs `inputmodule-control claude-hook` on its hook events. That writes
//! a small state file per session and exits right away, so Claude never waits for the
//! LED matrix. `inputmodule-control led-matrix --claude` reads the state files and
//! draws one bar per session:
//!
//! - Working: dim, with a light moving back and forth
//! - Idle, waiting for the next prompt: steady
//! - Needs attention, like a permission prompt or a question: blinking

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;
use serde_json::{json, Value};
use serialport::SerialPort;

use crate::inputmodule::{
    find_serialdevs, try_send_grey_frame, try_wake_and_wait, GreyGrid, HEIGHT, WIDTH,
};

/// Handle a Claude Code hook event, for showing it with `led-matrix --claude`.
///
/// Reads the hook's JSON from stdin. Add this to ~/.claude/settings.json:
///
/// {"hooks": {
///   "SessionStart":      [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "SessionEnd":        [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "UserPromptSubmit":  [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "PreToolUse":        [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "PostToolUse":       [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "PermissionRequest": [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "Notification":      [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}],
///   "Stop":              [{"hooks": [{"type": "command", "command": "inputmodule-control claude-hook", "timeout": 5}]}]
/// }}
#[derive(Parser, Debug)]
#[command(verbatim_doc_comment)]
pub struct ClaudeHookSubcommand {}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Working,
    Idle,
    Attention,
}

impl State {
    fn name(self) -> &'static str {
        match self {
            State::Working => "working",
            State::Idle => "idle",
            State::Attention => "attention",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "working" => Some(State::Working),
            "idle" => Some(State::Idle),
            "attention" => Some(State::Attention),
            _ => None,
        }
    }
}

/// Where the hooks leave the state of each session for the LED matrix program
fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("inputmodule-claude")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The Claude Code process and when it started, to tell when the session is gone
/// without SessionEnd, like when the terminal was closed.
/// The start time makes sure it's not another process that got the same PID later.
#[cfg(target_os = "linux")]
fn claude_process() -> Option<(u32, u64)> {
    // Claude Code runs hooks through a shell, which may or may not stay around
    let mut pid = std::os::unix::process::parent_id();
    for _ in 0..4 {
        let (comm, ppid, start) = proc_stat(pid)?;
        if !["sh", "bash", "dash", "zsh", "fish"].contains(&comm.as_str()) {
            return Some((pid, start));
        }
        pid = ppid;
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn claude_process() -> Option<(u32, u64)> {
    None
}

/// Name, parent PID and start time of a process
#[cfg(target_os = "linux")]
fn proc_stat(pid: u32) -> Option<(String, u32, u64)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The name is in parentheses and may contain spaces, the other fields come after it
    let (name, rest) = stat.split_once(" (")?.1.rsplit_once(") ")?;
    let fields: Vec<&str> = rest.split(' ').collect();
    // Fields 4 and 22 in `man proc_pid_stat`, counting from the state as field 3
    let ppid = fields.get(1)?.parse().ok()?;
    let start = fields.get(19)?.parse().ok()?;
    Some((name.to_string(), ppid, start))
}

fn process_alive(pid: u32, start: u64) -> bool {
    #[cfg(target_os = "linux")]
    {
        matches!(proc_stat(pid), Some((_, _, s)) if s == start)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (pid, start);
        true
    }
}

/// Without a process to check, forget sessions that haven't done anything for this long
const STALE_MS: u64 = 24 * 60 * 60 * 1000;

/// Run from a Claude Code hook. Never fails, so it can't get in Claude's way.
pub fn hook_cmd() {
    let mut input = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).is_err() {
        return;
    }
    let Ok(event) = serde_json::from_str::<Value>(&input) else {
        return;
    };
    let _ = handle_hook_event(&event);
}

fn handle_hook_event(event: &Value) -> Option<()> {
    let str_field = |name: &str| event.get(name).and_then(Value::as_str);
    // Only allow characters that are safe in a file name
    let session: String = str_field("session_id")?
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if session.is_empty() {
        return None;
    }
    let dir = state_dir();
    let path = dir.join(format!("{session}.json"));
    // Present when the event comes from a subagent
    let agent = str_field("agent_id");

    let new_state = match str_field("hook_event_name")? {
        "SessionEnd" => {
            let _ = fs::remove_file(&path);
            return Some(());
        }
        "SessionStart" | "Stop" => State::Idle,
        "UserPromptSubmit" | "PostToolUse" | "PostToolUseFailure" => State::Working,
        "PreToolUse" => match str_field("tool_name") {
            // Tools that ask the user something
            Some("AskUserQuestion" | "ExitPlanMode") => State::Attention,
            _ => State::Working,
        },
        "PermissionRequest" => State::Attention,
        "Notification" => match str_field("notification_type") {
            Some(
                "permission_prompt"
                | "elicitation_dialog"
                | "elicitation_url_dialog"
                | "agent_needs_input",
            ) => State::Attention,
            _ => return Some(()),
        },
        _ => return Some(()),
    };

    let old: Value = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let old_state = old["state"].as_str().and_then(State::from_name);
    let old_agent = old["attention_agent"].as_str();

    // While a subagent waits for the user, other subagents keep going.
    // Only the agent that asked, or the main agent, ends the attention.
    if old_state == Some(State::Attention)
        && new_state != State::Attention
        && agent.is_some()
        && agent != old_agent
    {
        return Some(());
    }

    let now = now_ms();
    let process = claude_process();
    let state = json!({
        "state": new_state.name(),
        // Keep the order of the bars when the state changes
        "started": old["started"].as_u64().unwrap_or(now),
        "updated": now,
        "pid": process.map(|p| p.0),
        "pid_start": process.map(|p| p.1),
        "attention_agent": if new_state == State::Attention { agent } else { None },
        "cwd": str_field("cwd"),
    });

    // Write to a temporary file and rename, so the LED matrix never reads half a file
    fs::create_dir_all(&dir).ok()?;
    let tmp = dir.join(format!(".{session}.tmp"));
    fs::write(&tmp, state.to_string()).ok()?;
    fs::rename(&tmp, &path).ok()
}

struct Session {
    state: State,
    started: u64,
}

/// Read the state of all sessions, oldest first.
/// Removes the files of sessions whose Claude Code process is gone.
fn read_sessions(dir: &Path) -> Vec<Session> {
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };
    let now = now_ms();
    let mut sessions = vec![];
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let Some(v) = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            continue;
        };
        let Some(state) = v["state"].as_str().and_then(State::from_name) else {
            continue;
        };
        let gone = match (v["pid"].as_u64(), v["pid_start"].as_u64()) {
            (Some(pid), Some(start)) => !process_alive(pid as u32, start),
            _ => now.saturating_sub(v["updated"].as_u64().unwrap_or(0)) > STALE_MS,
        };
        if gone {
            let _ = fs::remove_file(&path);
            continue;
        }
        sessions.push(Session {
            state,
            started: v["started"].as_u64().unwrap_or(0),
        });
    }
    sessions.sort_by_key(|s| s.started);
    sessions
}

/// Each session gets a bar this many LEDs high, with one dark row in between
const BAR_HEIGHT: usize = 2;
const MAX_BARS: usize = (HEIGHT + 1) / (BAR_HEIGHT + 1);

// The firmware has no gamma curve for greyscale, these are PWM values.
// Very low values look uneven on the hardware, so dim isn't too dim.
const WORKING_DIM: u8 = 24;
const WORKING_LIGHT: u8 = 180;
const IDLE: u8 = 90;
const ATTENTION: u8 = 255;

/// How long the working light takes to go across and back
const WORKING_PERIOD: f32 = 1.6;
/// How long attention blinks are on and off
const BLINK_PERIOD: f32 = 1.0;

fn render(sessions: &[Session], t: f32) -> GreyGrid {
    let mut grid = [[0; HEIGHT]; WIDTH];

    // If there are more sessions than fit, the ones that need attention go first
    let mut shown: Vec<&Session> = sessions
        .iter()
        .filter(|s| s.state == State::Attention)
        .chain(sessions.iter().filter(|s| s.state != State::Attention))
        .take(MAX_BARS)
        .collect();
    shown.sort_by_key(|s| s.started);

    // Triangle wave from 0 to WIDTH-1 and back
    let phase = (t / WORKING_PERIOD).fract();
    let light_x = ((1.0 - (2.0 * phase - 1.0).abs()) * (WIDTH - 1) as f32).round() as usize;
    let blink_on = (t / BLINK_PERIOD).fract() < 0.5;

    for (i, session) in shown.iter().enumerate() {
        let top = i * (BAR_HEIGHT + 1);
        for (x, col) in grid.iter_mut().enumerate() {
            let val = match session.state {
                State::Working if x == light_x => WORKING_LIGHT,
                State::Working => WORKING_DIM,
                State::Idle => IDLE,
                State::Attention if blink_on => ATTENTION,
                State::Attention => 0,
            };
            col[top..top + BAR_HEIGHT].fill(val);
        }
    }
    grid
}

/// Whether the picture changes over time
fn animating(sessions: &[Session]) -> bool {
    sessions.iter().any(|s| s.state != State::Idle)
}

/// Frames per second while something moves or blinks
const ANIMATION_FPS: u64 = 20;
/// How often to look for changes in the state files
const POLL_INTERVAL: Duration = Duration::from_millis(200);
/// The firmware goes to sleep after 60s without commands. Resend the picture before that.
const KEEPALIVE: Duration = Duration::from_secs(30);
/// How long to wait before looking for the LED matrix again, after losing it
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

fn open_ports(args: &crate::ClapCli) -> Vec<Box<dyn SerialPort>> {
    let (serialdevs, _) = find_serialdevs(args, false);
    serialdevs
        .iter()
        .filter_map(|dev| {
            // Long timeout, because the firmware doesn't service USB while fading in
            let mut port = serialport::new(dev, 115_200)
                .timeout(Duration::from_secs(10))
                .open()
                .ok()?;
            try_wake_and_wait(&mut port).ok()?;
            Some(port)
        })
        .collect()
}

/// Show the state of the Claude Code sessions until stopped
pub fn claude_cmd(args: &crate::ClapCli) {
    let dir = state_dir();
    println!("Showing Claude Code sessions from {}", dir.display());
    let start = Instant::now();
    let mut ports = vec![];
    let mut sessions = vec![];
    let mut last_poll: Option<Instant> = None;
    let mut last_frame: Option<GreyGrid> = None;
    let mut last_sent = Instant::now();

    loop {
        if ports.is_empty() {
            ports = open_ports(args);
            if ports.is_empty() {
                thread::sleep(RECONNECT_DELAY);
                continue;
            }
            // Draw it again on the new connection
            last_frame = None;
            last_sent = Instant::now();
        }

        if last_poll.is_none_or(|t| t.elapsed() >= POLL_INTERVAL) {
            sessions = read_sessions(&dir);
            last_poll = Some(Instant::now());
        }

        let frame = render(&sessions, start.elapsed().as_secs_f32());
        let blank = frame.iter().flatten().all(|&v| v == 0);
        // A blank picture can let the device sleep, that turns the LEDs off anyway
        let keepalive = !blank && last_sent.elapsed() >= KEEPALIVE;
        if last_frame != Some(frame) || keepalive {
            // After a minute without commands it's asleep, wait until it's awake again
            let asleep = last_sent.elapsed() >= Duration::from_secs(50);
            let result = ports.iter_mut().try_for_each(|port| {
                if asleep {
                    try_wake_and_wait(port)?;
                }
                try_send_grey_frame(port, &frame)
            });
            match result {
                Ok(()) => {
                    last_frame = Some(frame);
                    last_sent = Instant::now();
                }
                Err(err) => {
                    println!("Lost the LED matrix ({err}), looking for it again");
                    ports.clear();
                    thread::sleep(RECONNECT_DELAY);
                    continue;
                }
            }
        }

        let wait = if animating(&sessions) {
            Duration::from_millis(1000 / ANIMATION_FPS)
        } else {
            POLL_INTERVAL
        };
        thread::sleep(wait);
    }
}
