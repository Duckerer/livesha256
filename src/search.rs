// Подбор текста под префикс хеша: CPU (быстрый/универсальный) и GPU
use crate::gpu::{self, Gpu};
use crate::hashing::*;
use sha2::digest::generic_array::GenericArray;
use sha2::compress256;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Method {
    Gpu,
    CpuFast,
    CpuGeneric,
    Hybrid,
}

pub struct Ctl {
    pub stop: AtomicBool,
    pub tries: AtomicU64,
    pub cur: AtomicU64,
    pub best_len: AtomicUsize,
    pub best: Mutex<(usize, u64)>,
    pub err: Mutex<Option<String>>,
}

impl Ctl {
    fn new() -> Self {
        Ctl {
            stop: AtomicBool::new(false),
            tries: AtomicU64::new(0),
            cur: AtomicU64::new(u64::MAX),
            best_len: AtomicUsize::new(0),
            best: Mutex::new((0, 0)),
            err: Mutex::new(None),
        }
    }
    pub fn record(&self, m: usize, n: u64) {
        if m > self.best_len.load(Relaxed) {
            let mut b = self.best.lock().unwrap();
            if m > b.0 {
                *b = (m, n);
                self.best_len.store(m, Relaxed);
            }
        }
    }
}

pub struct Search {
    pub ctl: Arc<Ctl>,
    pub rx: mpsc::Receiver<String>,
    pub used: Method,
    pub note: String,
    pub dev: String,
    pub target: String,
    pub tlen: usize,
    pub done: bool,
    pub elapsed: f64,
    pub speed: f64,
    blen: usize,
    t0: Instant,
    last: (Instant, u64),
    base_hs: Hs,
}

pub const IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Метка CPU-попыток (старший бит): отличает десятичный суффикс CPU от hex-суффикса GPU
pub const TAG: u64 = 1 << 63;

/// Сколько нулей-«набивки» нужно, чтобы длина сообщения была 64*q + 52 (раскладка для GPU)
pub fn filler(blen: usize) -> usize {
    (35 - (blen as i64)).rem_euclid(64) as usize
}

pub fn sfx_for(blen: usize, n: u64) -> Vec<u8> {
    if n & TAG != 0 {
        let mut b = [0u8; 24];
        let l = suffix(n & !TAG, &mut b);
        b[..l].to_vec()
    } else {
        let mut v = vec![b'\n'];
        v.extend(std::iter::repeat(b'0').take(filler(blen)));
        v.extend_from_slice(format!("{:016x}", n).as_bytes());
        v
    }
}

impl Search {
    pub fn spawn(algo: Algo, base: String, target: &str, threads: usize, method: Method) -> Result<Search, String> {
        let target = target.trim().to_lowercase();
        if target.is_empty() || target.len() > algo.hex_len() || !target.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Нужен hex, 1-{} символов", algo.hex_len()));
        }
        let nib: Vec<u8> = target.chars().map(|c| c.to_digit(16).unwrap() as u8).collect();
        let tlen = nib.len();
        let mut val = [0u32; 8];
        for (i, &n) in nib.iter().enumerate() {
            val[i / 8] |= (n as u32) << (28 - 4 * (i % 8));
        }
        let mut note = String::new();
        let mut used = method;
        if algo != Algo::Sha256 && method != Method::CpuGeneric {
            used = Method::CpuGeneric;
            note = "Быстрый метод и GPU только для SHA-256 — взял универсальный CPU".into();
        }
        let ctl = Arc::new(Ctl::new());
        let (tx, rx) = mpsc::channel();
        let base = Arc::new(base);
        let blen = base.len();
        let mut hs = Hs::new(algo);
        hs.update(base.as_bytes());
        let mut dev = String::new();
        let threads = threads.max(1);

        let mut gpu_obj = None;
        if used == Method::Gpu || used == Method::Hybrid {
            match Gpu::new() {
                Ok(g) => {
                    dev = g.name.clone();
                    gpu_obj = Some(g);
                }
                Err(e) => {
                    used = Method::CpuFast;
                    note = format!("GPU не взлетел ({}), ушёл на CPU", e);
                }
            }
        }

        if let Some(g) = gpu_obj {
            let (base, ctl, tx) = (base.clone(), ctl.clone(), tx.clone());
            let target = target.clone();
            thread::spawn(move || gpu::worker(g, base, val, tlen, target, ctl, tx));
        }
        if used == Method::Hybrid && algo == Algo::Sha256 || used == Method::CpuFast {
            for t in 0..threads {
                let (base, ctl, tx) = (base.clone(), ctl.clone(), tx.clone());
                thread::spawn(move || fast_worker(t, threads, base, val, tlen, ctl, tx));
            }
        } else if used == Method::CpuGeneric {
            let nib = Arc::new(nib);
            for t in 0..threads {
                let (base, ctl, tx, nib, hs) = (base.clone(), ctl.clone(), tx.clone(), nib.clone(), hs.clone());
                thread::spawn(move || gen_worker(t, threads, hs, base, nib, ctl, tx));
            }
        }
        let now = Instant::now();
        Ok(Search {
            ctl,
            rx,
            used,
            note,
            dev,
            target,
            tlen,
            done: false,
            elapsed: 0.0,
            speed: 0.0,
            blen,
            t0: now,
            last: (now, 0),
            base_hs: hs,
        })
    }

    pub fn tick(&mut self) {
        if self.done {
            return;
        }
        self.elapsed = self.t0.elapsed().as_secs_f64();
        let now = Instant::now();
        let dt = now.duration_since(self.last.0).as_secs_f64();
        if dt >= 0.5 {
            let tr = self.ctl.tries.load(Relaxed);
            let inst = tr.saturating_sub(self.last.1) as f64 / dt;
            self.speed = if self.speed == 0.0 { inst } else { self.speed * 0.5 + inst * 0.5 };
            self.last = (now, tr);
        }
    }

    pub fn finish(&mut self) {
        if !self.done {
            self.elapsed = self.t0.elapsed().as_secs_f64();
            self.done = true;
            self.ctl.stop.store(true, Relaxed);
            self.speed = self.ctl.tries.load(Relaxed) as f64 / self.elapsed.max(0.001);
        }
    }

    pub fn hash_for(&self, n: u64) -> String {
        let mut h = self.base_hs.clone();
        h.update(&sfx_for(self.blen, n));
        hex(&h.finish())
    }
}

fn lead_words(st: &[u32; 8], val: &[u32; 8], tlen: usize) -> usize {
    let mut m = 0usize;
    for w in 0..8 {
        let x = st[w] ^ val[w];
        if x == 0 {
            m += 8;
        } else {
            m += (x.leading_zeros() / 4) as usize;
            break;
        }
    }
    m.min(tlen)
}

fn nib_lead(d: &[u8], nib: &[u8]) -> usize {
    for (i, &nb) in nib.iter().enumerate() {
        let b = d[i / 2];
        let v = if i % 2 == 0 { b >> 4 } else { b & 15 };
        if v != nb {
            return i;
        }
    }
    nib.len()
}

fn block(b: &[u8]) -> &[GenericArray<u8, sha2::digest::consts::U64>] {
    std::slice::from_ref(GenericArray::from_slice(b))
}

/// Быстрый CPU-метод: midstate + один-два блока SHA-256 напрямую, без аллокаций
fn fast_worker(t: usize, threads: usize, base: Arc<String>, val: [u32; 8], tlen: usize, ctl: Arc<Ctl>, tx: mpsc::Sender<String>) {
    let bytes = base.as_bytes();
    let blen = bytes.len();
    let full = blen / 64;
    let mut st0 = IV;
    for i in 0..full {
        compress256(&mut st0, block(&bytes[i * 64..i * 64 + 64]));
    }
    let tl = blen - full * 64;
    let mut buf = [0u8; 128];
    buf[..tl].copy_from_slice(&bytes[full * 64..]);
    let mut s = [0u8; 24];
    let mut n = t as u64;
    let mut best = 0usize;
    let mut cnt = 0u32;
    loop {
        let l = suffix(n, &mut s);
        buf[tl..tl + l].copy_from_slice(&s[..l]);
        let p = tl + l;
        buf[p] = 0x80;
        let nb = (p + 9 + 63) / 64;
        let end = nb * 64;
        buf[p + 1..end].fill(0);
        buf[end - 8..end].copy_from_slice(&(((blen + l) as u64) * 8).to_be_bytes());
        let mut st = st0;
        for b in 0..nb {
            compress256(&mut st, block(&buf[b * 64..b * 64 + 64]));
        }
        let m0 = ((st[0] ^ val[0]).leading_zeros() / 4) as usize;
        if m0 > best || m0 == 8 {
            let m = lead_words(&st, &val, tlen);
            if m > best {
                best = m;
                ctl.record(m, n | TAG);
                if m >= tlen {
                    ctl.stop.store(true, Relaxed);
                    let _ = tx.send(format!("{}{}", base, String::from_utf8_lossy(&s[..l])));
                    return;
                }
            }
        }
        n += threads as u64;
        cnt += 1;
        if cnt & 0x3FFF == 0 {
            ctl.tries.fetch_add(0x4000, Relaxed);
            if t == 0 {
                ctl.cur.store(n | TAG, Relaxed);
            }
            if ctl.stop.load(Relaxed) {
                return;
            }
        }
    }
}

/// Универсальный CPU-метод: любой алгоритм, клон состояния после базового текста
fn gen_worker(t: usize, threads: usize, hs: Hs, base: Arc<String>, nib: Arc<Vec<u8>>, ctl: Arc<Ctl>, tx: mpsc::Sender<String>) {
    let tlen = nib.len();
    let mut out = [0u8; 64];
    let mut s = [0u8; 24];
    let mut n = t as u64;
    let mut best = 0usize;
    let mut cnt = 0u32;
    loop {
        let l = suffix(n, &mut s);
        let mut h = hs.clone();
        h.update(&s[..l]);
        let ol = h.finish_into(&mut out);
        let m = nib_lead(&out[..ol], &nib);
        if m > best {
            best = m;
            ctl.record(m, n | TAG);
            if m >= tlen {
                ctl.stop.store(true, Relaxed);
                let _ = tx.send(format!("{}{}", base, String::from_utf8_lossy(&s[..l])));
                return;
            }
        }
        n += threads as u64;
        cnt += 1;
        if cnt & 0x1FFF == 0 {
            ctl.tries.fetch_add(0x2000, Relaxed);
            if t == 0 {
                ctl.cur.store(n | TAG, Relaxed);
            }
            if ctl.stop.load(Relaxed) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_for_matches_real_text() {
        for base in ["", "duck", "x".repeat(70).as_str()] {
            let mut s = Search::spawn(Algo::Sha256, base.to_string(), "ffffffff", 1, Method::CpuGeneric).unwrap();
            s.finish();
            // GPU-раскладка (без метки)
            let n = 0x1234_5678_9abcu64;
            let mut t = base.as_bytes().to_vec();
            t.extend(sfx_for(base.len(), n));
            assert_eq!((t.len()) % 64, 52, "длина должна быть 64q+52");
            assert_eq!(s.hash_for(n), hash_hex(Algo::Sha256, &t));
            // CPU-раскладка (с меткой)
            let mut t2 = base.as_bytes().to_vec();
            t2.extend(format!("\n{}", 777).bytes());
            assert_eq!(s.hash_for(777 | TAG), hash_hex(Algo::Sha256, &t2));
        }
    }
}
