//! Transcribe audio/video with Groq Whisper and write one `.srt`.
//!
//! Pipeline, built to send one request whenever the audio allows it:
//!   1. Demux-only probe (no decode): true duration = audio packets × frame size
//!      / sample rate. Container headers and packet timestamps can both lie
//!      (a file reporting 81.5 min carried 88.1 min); the packet count did not.
//!   2. Pick the bitrate from the duration: the highest rate ≤ 48 kbps whose
//!      output fits the byte budget, never below 32 kbps.
//!   3. Encode once to mono 16 kHz mp3. If it fits, that is the only request.
//!      If not, split the mp3 with `-c copy` (seconds, no re-encode).
//!   4. Transcribe chunks in parallel with retries, merge on offsets measured
//!      from each chunk file, trim a degenerate tail, write the SRT.
//!
//! Work dir is deterministic (keyed on input path, size, mtime, model,
//! language), so a re-run after a failure resumes instead of re-paying.

use clap::Parser;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::thread;
use std::time::{Duration, Instant};

const API: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// Groq states the limit as 25 MB, decimal.
const MAX_BYTES: u64 = 25_000_000;
const MAX_KBPS: u32 = 48;
/// Floor for automatic bitrate choice. Below this, chunk instead.
const FLOOR_KBPS: u32 = 32;

#[derive(Parser)]
#[command(about = "Transcribe audio/video to SRT with Groq Whisper (one request when it fits)")]
struct Args {
    /// Input audio/video file. With --from-json, a merged/per-chunk JSON.
    input: PathBuf,
    /// ISO-639-1 code; `auto` lets the API detect per chunk.
    #[arg(long, default_value = "es")]
    language: String,
    #[arg(long, default_value = "whisper-large-v3")]
    model: String,
    /// Target MB per request (API limit is 25, decimal).
    #[arg(long, default_value_t = 24.0)]
    budget_mb: f64,
    /// Mono mp3 kbps, or `auto` (≤48, ≥32, chosen to fit one request).
    #[arg(long, default_value = "auto")]
    bitrate: String,
    #[arg(long, default_value_t = 3)]
    workers: usize,
    /// Whisper prompt. Not recommended: even a names-only prompt dropped 37%
    /// of words on a measured sample. Fix names in the output instead.
    #[arg(long)]
    prompt: Option<String>,
    /// No-op, kept for old command lines (there is no automatic prompt).
    #[arg(long, hide = true)]
    no_auto_prompt: bool,
    /// Keep both --language and an explicit --prompt (degrades output).
    #[arg(long)]
    allow_language_prompt: bool,
    #[arg(long)]
    outdir: Option<PathBuf>,
    /// Keep chunk audio and JSON (incl. merged transcript.json).
    #[arg(long)]
    keep_work: bool,
    /// Re-transcribe chunks even if valid JSON exists.
    #[arg(long)]
    overwrite: bool,
    /// Keep trailing 1–2 word segments (silence/applause filler).
    #[arg(long)]
    keep_tail: bool,
    /// Convert an existing JSON instead of transcribing.
    #[arg(long)]
    from_json: bool,
    /// Output format: srt, vtt or txt.
    #[arg(long, default_value = "srt")]
    format: String,
}

macro_rules! log { ($($t:tt)*) => { eprintln!($($t)*) } }

fn die(msg: impl AsRef<str>) -> ! {
    eprintln!("error: {}", msg.as_ref());
    exit(1)
}

fn run(cmd: &mut Command) -> (bool, String, String) {
    match cmd.output() {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        ),
        Err(e) => (false, String::new(), e.to_string()),
    }
}

fn which(tool: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
        .unwrap_or(false)
}

// ---------------------------------------------------------------- probing

struct Probe {
    container: f64,
    /// Packet-count duration; None when the codec has no fixed frame size.
    packets: Option<f64>,
}

fn probe_input(src: &Path) -> Probe {
    let (ok, out, err) = run(Command::new("ffprobe").args([
        "-v", "error", "-select_streams", "a:0", "-count_packets",
        "-show_entries", "stream=codec_name,profile,sample_rate,nb_read_packets:format=duration",
        "-of", "json",
    ]).arg(src));
    if !ok {
        die(format!("ffprobe failed on {}: {}", src.display(), err.trim()));
    }
    let v: Value = serde_json::from_str(&out).unwrap_or(Value::Null);
    let stream = v["streams"].get(0).unwrap_or_else(|| {
        die(format!("{} has no audio stream", src.display()))
    });
    let num = |x: &Value| -> Option<f64> {
        x.as_f64().or_else(|| x.as_str().and_then(|s| s.parse().ok()))
    };
    let container = num(&v["format"]["duration"]).unwrap_or(0.0);
    let sr = num(&stream["sample_rate"]).unwrap_or(0.0);
    // ffprobe does not report frame_size; it is fixed per codec. Variable-
    // frame codecs (opus, vorbis, flac, pcm) fall back to the container and
    // rely on the post-encode measurement.
    let profile = stream["profile"].as_str().unwrap_or("");
    let fs_ = match stream["codec_name"].as_str().unwrap_or("") {
        "aac" if profile.starts_with("HE") => 2048.0,
        "aac" => 1024.0,
        "mp3" if sr >= 32000.0 => 1152.0,
        "mp3" => 576.0,
        "ac3" | "eac3" => 1536.0,
        _ => 0.0,
    };
    let n = num(&stream["nb_read_packets"]).unwrap_or(0.0);
    let packets = (sr > 0.0 && fs_ > 0.0 && n > 0.0).then(|| n * fs_ / sr);
    Probe { container, packets }
}

fn file_duration(p: &Path) -> f64 {
    let (ok, out, err) = run(Command::new("ffprobe").args([
        "-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nokey=1",
    ]).arg(p));
    if !ok {
        die(format!("ffprobe failed on {}: {}", p.display(), err.trim()));
    }
    out.trim().parse().unwrap_or_else(|_| die(format!("no duration for {}", p.display())))
}

// ---------------------------------------------------------------- encoding

/// Bitrates libmp3lame accepts at 16 kHz (MPEG-2 layer III). Anything else is
/// silently snapped: "36k" encodes as 32k, "39k" as 40k (and overshoots).
const MPEG2_KBPS: &[u32] = &[8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

fn snap_down(kbps: u32) -> u32 {
    MPEG2_KBPS.iter().copied().filter(|&r| r <= kbps).max().unwrap_or(8)
}

/// Highest valid rate ≤ 48 that fits `duration` in `budget` bytes.
fn kbps_for(duration: f64, budget: f64) -> u32 {
    snap_down(((budget * 8.0 / duration / 1000.0).floor() as u32).min(MAX_KBPS))
}

fn encode(src: &Path, out: &Path, kbps: u32) {
    let t = Instant::now();
    let (ok, _, err) = run(Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"]).arg(src)
        .args(["-vn", "-ac", "1", "-ar", "16000", "-b:a", &format!("{kbps}k")])
        .arg(out));
    if !ok || !out.exists() {
        // Broken-timestamp sources spam "non monotonically increasing dts";
        // show only the tail.
        let tail: Vec<&str> = err.lines().rev().take(5).collect();
        die(format!("ffmpeg encode failed: {}", tail.into_iter().rev().collect::<Vec<_>>().join("\n")));
    }
    log!("  encoded {kbps} kbps in {:.1}s", t.elapsed().as_secs_f64());
}

/// Produce request-sized mp3s in `work`. Returns them in order.
fn prepare_chunks(src: &Path, work: &Path, est: f64, args: &Args) -> Vec<PathBuf> {
    let mut existing: Vec<PathBuf> = fs::read_dir(work).into_iter().flatten().flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().map_or(false, |n| n.to_string_lossy().starts_with("part-")) && p.extension().map_or(false, |e| e == "mp3"))
        .collect();
    if !existing.is_empty() {
        existing.sort();
        log!("  reusing {} chunk(s) from {}", existing.len(), work.display());
        return existing;
    }
    let budget = args.budget_mb * 1e6;
    let audio = work.join("audio.mp3");
    let mut kbps = match args.bitrate.as_str() {
        "auto" => kbps_for(est, budget).max(FLOOR_KBPS),
        s => {
            let k: u32 = s.parse().unwrap_or_else(|_| die("--bitrate must be a number or `auto`"));
            let snapped = snap_down(k);
            if snapped != k {
                log!("  --bitrate {k} is not a valid 16 kHz mp3 rate; using {snapped}");
            }
            snapped
        }
    };
    log!("  {:.1} min -> {kbps} kbps (fits ~{:.0} min per request)", est / 60.0, budget * 8.0 / (kbps as f64 * 1000.0) / 60.0);
    encode(src, &audio, kbps);
    let mut size = fs::metadata(&audio).unwrap().len();
    let mut dur = file_duration(&audio);

    // The estimate was wrong (no packet count, odd codec) and a re-encode at a
    // still-acceptable rate would make it one request: worth one more pass.
    if size as f64 > budget && args.bitrate == "auto" {
        let need = kbps_for(dur, budget);
        if need >= FLOOR_KBPS && need < kbps {
            log!("  {:.2} MB over budget; re-encoding at {need} kbps for a single request", size as f64 / 1e6);
            kbps = need;
            encode(src, &audio, kbps);
            size = fs::metadata(&audio).unwrap().len();
            dur = file_duration(&audio);
        }
    }

    if size <= MAX_BYTES && size as f64 <= budget * 1.02 {
        let one = work.join("part-000.mp3");
        fs::rename(&audio, &one).unwrap();
        log!("  1 request: {:.1} MB, {:.1} min at {kbps} kbps", size as f64 / 1e6, dur / 60.0);
        return vec![one];
    }

    // Split without re-encoding. Equal-length pieces, each under budget.
    let n = (size as f64 / budget).ceil().max(2.0);
    let seg = (dur / n).ceil() + 1.0;
    let (ok, _, err) = run(Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"]).arg(&audio)
        .args(["-c", "copy", "-f", "segment", "-segment_time", &format!("{seg}"), "-reset_timestamps", "1"])
        .arg(work.join("part-%03d.mp3")));
    if !ok {
        die(format!("ffmpeg split failed: {}", err.trim()));
    }
    let _ = fs::remove_file(&audio);
    let mut parts: Vec<PathBuf> = fs::read_dir(work).unwrap().flatten().map(|e| e.path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("part-"))
        .collect();
    parts.sort();
    let biggest = parts.iter().map(|p| fs::metadata(p).unwrap().len()).max().unwrap_or(0);
    if biggest > MAX_BYTES {
        die(format!("largest chunk {:.2} MB exceeds the 25 MB limit; lower --budget-mb", biggest as f64 / 1e6));
    }
    log!("  {} requests at {kbps} kbps, largest {:.1} MB", parts.len(), biggest as f64 / 1e6);
    parts
}

// ---------------------------------------------------------------- requests

fn valid_json(p: &Path) -> bool {
    fs::read_to_string(p).ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .map_or(false, |v| v.get("error").is_none() && v["segments"].is_array())
}

fn transcribe_chunk(chunk: &Path, out: &Path, args: &Args, prompt: Option<&str>, key: &str) -> Result<&'static str, String> {
    if !args.overwrite && valid_json(out) {
        return Ok("cached");
    }
    let mut err = String::from("no attempt made");
    for attempt in 1..=3u32 {
        let mut cmd = Command::new("curl");
        cmd.args(["-sS", "--max-time", "900", API, "-H"])
            .arg(format!("Authorization: Bearer {key}"))
            .args(["-F", &format!("model={}", args.model)])
            .arg("-F").arg(format!("file=@{}", chunk.display()))
            .args(["-F", "temperature=0", "-F", "response_format=verbose_json"]);
        if args.language != "auto" {
            cmd.args(["-F", &format!("language={}", args.language)]);
        }
        if let Some(p) = prompt {
            // `-F prompt=<x>` would treat a leading @ or < as a file reference.
            cmd.arg("--form-string").arg(format!("prompt={p}"));
        }
        let (_, body, stderr) = run(&mut cmd);
        let body = body.trim();
        if body.is_empty() {
            err = if stderr.trim().is_empty() { "empty response".into() } else { stderr.trim().chars().take(200).collect() };
        } else {
            match serde_json::from_str::<Value>(body) {
                Ok(v) if v.get("error").is_none() && v["segments"].is_array() => {
                    fs::write(out, body).map_err(|e| e.to_string())?;
                    return Ok("ok");
                }
                Ok(v) => err = v.get("error").unwrap_or(&v).to_string().chars().take(200).collect(),
                Err(_) => err = body.chars().take(200).collect(),
            }
        }
        if attempt < 3 {
            thread::sleep(Duration::from_secs(2u64.pow(attempt)));
        }
    }
    Err(err)
}

// ---------------------------------------------------------------- prompt

// There is deliberately no automatic prompt. A hint derived from the path
// ("Conferencia Economía Digital conferences economics") made Whisper echo it
// for whole minutes of a 69-min talk: 82 -> 14 segments on a 300s sample.

fn mmss(t: f64) -> String {
    let t = t.max(0.0) as u64;
    format!("{}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)
}

/// Speech runs ~2-3 words/s. A long cue with almost no words, or one text
/// repeated across many cues, is a decoding loop, not speech.
fn check_loops(segs: &[Seg]) {
    let sparse: Vec<&Seg> = segs.iter().filter(|s| {
        let d = s.end - s.start;
        d >= 10.0 && (s.text.split_whitespace().count() as f64) / d < 0.3
    }).collect();
    if sparse.len() >= 3 {
        let secs: f64 = sparse.iter().map(|s| s.end - s.start).sum();
        let at: Vec<String> = sparse.iter().take(5).map(|s| format!("{} {:?}", mmss(s.start), s.text)).collect();
        log!("WARNING: {} long cue(s) with <0.3 words/s cover {:.1} min — likely a hallucination loop. First: {}",
            sparse.len(), secs / 60.0, at.join("; "));
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for s in segs {
        let k: String = s.text.to_lowercase().chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
        let k = k.split_whitespace().collect::<Vec<_>>().join(" ");
        if !k.is_empty() {
            *counts.entry(k).or_default() += 1;
        }
    }
    if let Some((text, n)) = counts.into_iter().max_by_key(|(_, n)| *n) {
        if n >= 5 && n * 10 > segs.len() {
            log!("WARNING: {n}/{} cues are the same text {text:?} — likely a hallucination loop.", segs.len());
        }
    }
}

// ---------------------------------------------------------------- merge & output

struct Seg { start: f64, end: f64, text: String }

fn merge(chunks: &[PathBuf], jsons: &[PathBuf]) -> (Vec<Seg>, Vec<Option<String>>, Vec<usize>) {
    let mut segs = Vec::new();
    let mut langs = Vec::new();
    let mut boundaries = Vec::new();
    let mut offset = 0.0;
    for (chunk, jf) in chunks.iter().zip(jsons) {
        let v: Value = serde_json::from_str(&fs::read_to_string(jf).unwrap()).unwrap();
        langs.push(v["language"].as_str().map(String::from));
        if !segs.is_empty() {
            boundaries.push(segs.len());
        }
        for s in v["segments"].as_array().unwrap() {
            let start = s["start"].as_f64().unwrap_or(0.0) + offset;
            let end = s["end"].as_f64().unwrap_or(0.0) + offset;
            segs.push(Seg { start, end, text: s["text"].as_str().unwrap_or("").trim().to_string() });
        }
        // Offset from the real chunk length, not the API-reported duration.
        offset += if chunks.len() > 1 { file_duration(chunk) } else { 0.0 };
    }
    (segs, langs, boundaries)
}

fn ts(sec: f64, sep: char) -> String {
    let ms = (sec.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}:{:02}{sep}{:03}", ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000)
}

fn render(segs: &[Seg], fmt: &str) -> String {
    let mut o = String::new();
    match fmt {
        "txt" => {
            o = segs.iter().map(|s| s.text.as_str()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join(" ");
            o.push('\n');
        }
        "vtt" => {
            o.push_str("WEBVTT\n\n");
            for s in segs {
                o.push_str(&format!("{} --> {}\n{}\n\n", ts(s.start, '.'), ts(s.end, '.'), s.text));
            }
        }
        "srt" => {
            for (i, s) in segs.iter().enumerate() {
                o.push_str(&format!("{}\n{} --> {}\n{}\n\n", i + 1, ts(s.start, ','), ts(s.end, ','), s.text));
            }
        }
        f => die(format!("unknown format {f:?}; use srt, vtt or txt")),
    }
    o
}

fn lang_name(code: &str) -> &str {
    match code {
        "en" => "English", "es" => "Spanish", "pt" => "Portuguese",
        "fr" => "French", "de" => "German", "it" => "Italian", c => c,
    }
}

/// Fix what is safe to fix, warn about what is not.
fn validate(segs: &mut Vec<Seg>, langs: &[Option<String>], args: &Args) {
    if segs.is_empty() {
        log!("warning: no segments returned");
        return;
    }
    let boiler = Regex::new(
        r"(?i)^(speaker\s*\d+(\s*(,|and)\s*\d+)?\.?|gracias por ver.*|subt[ií]tulos (realizados|por).*|thank you for watching.*|amara\.org.*|www\..*)$",
    ).unwrap();
    check_loops(segs);
    let bad = segs.iter().filter(|s| boiler.is_match(&s.text)).count();
    if bad * 20 > segs.len() {
        log!("warning: {bad}/{} segments are boilerplate such as 'Speaker 1' — the prompt + language pair is the usual cause. Inspect before trusting.", segs.len());
    }
    if args.language != "auto" {
        let want = lang_name(&args.language);
        let wrong: Vec<String> = langs.iter().enumerate()
            .filter_map(|(i, l)| l.as_deref().filter(|l| *l != want).map(|l| format!("chunk {} = {l}", i + 1)))
            .collect();
        if !wrong.is_empty() {
            log!("warning: --language {} passed but the API reports {}", args.language, wrong.join(", "));
        }
    }
    // Whisper occasionally returns end < start; clamp so players accept it.
    let mut fixed = 0;
    for s in segs.iter_mut() {
        if s.end <= s.start {
            s.end = s.start + 0.5;
            fixed += 1;
        }
    }
    if fixed > 0 {
        log!("  clamped {fixed} segment(s) with zero/negative duration to 0.5s");
    }
    // Trailing 1–2 word cues are silence/applause filler ("y", "y", "gracias").
    let mut trimmed = Vec::new();
    while !args.keep_tail && trimmed.len() < 10 && segs.len() > 1
        && segs.last().map_or(false, |s| s.text.split_whitespace().count() <= 2) {
        trimmed.push(segs.pop().unwrap().text);
    }
    if !trimmed.is_empty() {
        trimmed.reverse();
        log!("  trimmed {} degenerate tail segment(s): {:?} (--keep-tail to keep)", trimmed.len(), trimmed);
    }
}

// ---------------------------------------------------------------- main

fn work_dir(src: &Path, args: &Args) -> PathBuf {
    let meta = fs::metadata(src).unwrap();
    let mut h = DefaultHasher::new();
    src.hash(&mut h);
    meta.len().hash(&mut h);
    meta.modified().ok().hash(&mut h);
    args.model.hash(&mut h);
    args.language.hash(&mut h);
    args.bitrate.hash(&mut h);
    std::env::temp_dir().join(format!("groq-asr-{:016x}", h.finish()))
}

fn from_json(args: &Args) {
    let s = fs::read_to_string(&args.input).unwrap_or_else(|e| die(e.to_string()));
    let v: Value = serde_json::from_str(&s).unwrap_or_else(|e| die(e.to_string()));
    if let Some(e) = v.get("error") {
        die(format!("input is an API error payload: {e}"));
    }
    let segs: Vec<Seg> = v["segments"].as_array().unwrap_or(&vec![]).iter().map(|s| Seg {
        start: s["start"].as_f64().unwrap_or(0.0),
        end: s["end"].as_f64().unwrap_or(0.0),
        text: s["text"].as_str().unwrap_or("").trim().to_string(),
    }).collect();
    use std::io::Write;
    // Piping into `head` closes stdout early; that is not an error.
    let _ = std::io::stdout().lock().write_all(render(&segs, &args.format).as_bytes());
}

fn main() {
    let args = Args::parse();
    if args.from_json {
        return from_json(&args);
    }
    let t0 = Instant::now();
    let key = std::env::var("GROQ_API_KEY").unwrap_or_default();
    if key.is_empty() {
        die("GROQ_API_KEY is not set (it lives in ~/.config/zsh/.zshenv)");
    }
    for tool in ["ffmpeg", "ffprobe", "curl"] {
        if !which(tool) {
            die(format!("{tool} not found in PATH"));
        }
    }
    let src = fs::canonicalize(&args.input).unwrap_or_else(|_| die(format!("no such file: {}", args.input.display())));

    let prompt = args.prompt.clone();
    if prompt.is_some() {
        log!("  warning: --prompt measured harmful even with names only (664 -> 416 words); check the output");
    }
    // Measured: language=<wrong> + content prompt -> 162 words vs 483.
    if args.language != "auto" && prompt.is_some() && !args.allow_language_prompt {
        die(format!("--language {} and --prompt together reliably degrade transcription (483 -> 162 words on a measured sample). Pass one, use --language auto, or --allow-language-prompt.", args.language));
    }

    let probe = probe_input(&src);
    let est = probe.packets.unwrap_or(probe.container);
    if probe.packets.map_or(false, |p| (p - probe.container).abs() > 1.5) {
        log!("warning: container says {:.1} min but the audio is {:.1} min. The SRT follows the audio; a player trusting the header may drift against it.",
            probe.container / 60.0, est / 60.0);
    }

    let outdir = args.outdir.clone().unwrap_or_else(|| src.parent().unwrap().to_path_buf());
    fs::create_dir_all(&outdir).unwrap();
    let work = work_dir(&src, &args);
    fs::create_dir_all(&work).unwrap();
    log!("{}: {:.1} min; work dir {}", src.file_name().unwrap().to_string_lossy(), est / 60.0, work.display());

    let chunks = prepare_chunks(&src, &work, est, &args);
    let jsons: Vec<PathBuf> = chunks.iter().map(|c| c.with_extension("json")).collect();

    let t_req = Instant::now();
    let pairs: Vec<(&PathBuf, &PathBuf)> = chunks.iter().zip(&jsons).collect();
    let results: Vec<(String, Result<&str, String>)> = thread::scope(|s| {
        let mut out = Vec::new();
        for batch in pairs.chunks(args.workers.max(1)) {
            let hs: Vec<_> = batch.iter().map(|&(c, j)| {
                let (args, prompt, key) = (&args, prompt.as_deref(), key.as_str());
                s.spawn(move || (c.file_name().unwrap().to_string_lossy().into_owned(), transcribe_chunk(c, j, args, prompt, key)))
            }).collect();
            out.extend(hs.into_iter().map(|h| h.join().unwrap()));
        }
        out
    });
    let mut failed = Vec::new();
    for (name, r) in &results {
        match r {
            Ok(st) => log!("  {name}: {st}"),
            Err(e) => { log!("  {name}: failed: {e}"); failed.push(name.clone()); }
        }
    }
    if !failed.is_empty() {
        die(format!("{} chunk(s) failed; re-run the same command to resume from {}", failed.len(), work.display()));
    }
    log!("  transcription: {:.1}s", t_req.elapsed().as_secs_f64());

    let (mut segs, langs, boundaries) = merge(&chunks, &jsons);
    for b in &boundaries {
        let (a, z) = (&segs[b - 1], &segs[*b]);
        log!("  boundary @ {}: …{:?} | {:?}…", ts(z.start, ','), a.text, z.text);
    }
    validate(&mut segs, &langs, &args);
    if let Some(last) = segs.last() {
        if (last.end - est).abs() > 300.0 && last.end > est {
            log!("warning: transcript ends at {} but audio is {:.1} min", ts(last.end, ','), est / 60.0);
        }
    }

    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = outdir.join(format!("{stem}.{}", args.format));
    fs::write(&out, render(&segs, &args.format)).unwrap();
    log!("{} segments, {:.1} min, {} request(s), {:.1}s total -> {}",
        segs.len(), est / 60.0, chunks.len(), t0.elapsed().as_secs_f64(), out.display());

    if args.keep_work {
        let merged = json!({
            "segments": segs.iter().enumerate().map(|(i, s)| json!({"id": i, "start": s.start, "end": s.end, "text": s.text})).collect::<Vec<_>>(),
            "language": args.language, "chunk_languages": langs, "duration": est,
            "model": args.model, "chunks": chunks.len(),
        });
        fs::write(work.join("transcript.json"), serde_json::to_string_pretty(&merged).unwrap()).unwrap();
        log!("kept intermediates in {} (transcript.json = merged)", work.display());
    } else {
        let _ = fs::remove_dir_all(&work);
    }
}
