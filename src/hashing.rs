// Алгоритмы, форматирование, общие хелперы
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use sha3::Sha3_256;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Algo {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sha3,
    Blake3,
    Crc32,
}

pub const ALGOS: [Algo; 8] = [
    Algo::Md5,
    Algo::Sha1,
    Algo::Sha256,
    Algo::Sha384,
    Algo::Sha512,
    Algo::Sha3,
    Algo::Blake3,
    Algo::Crc32,
];

impl Algo {
    pub fn name(self) -> &'static str {
        match self {
            Algo::Md5 => "MD5",
            Algo::Sha1 => "SHA-1",
            Algo::Sha256 => "SHA-256",
            Algo::Sha384 => "SHA-384",
            Algo::Sha512 => "SHA-512",
            Algo::Sha3 => "SHA3-256",
            Algo::Blake3 => "BLAKE3",
            Algo::Crc32 => "CRC32",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Algo::Md5 => "md5",
            Algo::Sha1 => "sha1",
            Algo::Sha256 => "sha256",
            Algo::Sha384 => "sha384",
            Algo::Sha512 => "sha512",
            Algo::Sha3 => "sha3",
            Algo::Blake3 => "blake3",
            Algo::Crc32 => "crc32",
        }
    }
    pub fn from_key(k: &str) -> Option<Algo> {
        ALGOS.iter().copied().find(|a| a.key() == k)
    }
    pub fn hex_len(self) -> usize {
        match self {
            Algo::Md5 => 32,
            Algo::Sha1 => 40,
            Algo::Sha256 | Algo::Sha3 | Algo::Blake3 => 64,
            Algo::Sha384 => 96,
            Algo::Sha512 => 128,
            Algo::Crc32 => 8,
        }
    }
}

#[derive(Clone)]
pub enum Hs {
    Md5(Md5),
    Sha1(Sha1),
    S256(Sha256),
    S384(Sha384),
    S512(Sha512),
    S3(Sha3_256),
    B3(Box<blake3::Hasher>),
    Crc(crc32fast::Hasher),
}

impl Hs {
    pub fn new(a: Algo) -> Self {
        match a {
            Algo::Md5 => Hs::Md5(Md5::new()),
            Algo::Sha1 => Hs::Sha1(Sha1::new()),
            Algo::Sha256 => Hs::S256(Sha256::new()),
            Algo::Sha384 => Hs::S384(Sha384::new()),
            Algo::Sha512 => Hs::S512(Sha512::new()),
            Algo::Sha3 => Hs::S3(Sha3_256::new()),
            Algo::Blake3 => Hs::B3(Box::new(blake3::Hasher::new())),
            Algo::Crc32 => Hs::Crc(crc32fast::Hasher::new()),
        }
    }

    pub fn update(&mut self, d: &[u8]) {
        match self {
            Hs::Md5(h) => h.update(d),
            Hs::Sha1(h) => h.update(d),
            Hs::S256(h) => h.update(d),
            Hs::S384(h) => h.update(d),
            Hs::S512(h) => h.update(d),
            Hs::S3(h) => h.update(d),
            Hs::B3(h) => {
                h.update(d);
            }
            Hs::Crc(h) => h.update(d),
        }
    }

    /// Результат без аллокаций (буфер 64 байта хватает на всё)
    pub fn finish_into(self, out: &mut [u8; 64]) -> usize {
        macro_rules! d {
            ($h:expr) => {{
                let r = $h.finalize();
                out[..r.len()].copy_from_slice(&r);
                r.len()
            }};
        }
        match self {
            Hs::Md5(h) => d!(h),
            Hs::Sha1(h) => d!(h),
            Hs::S256(h) => d!(h),
            Hs::S384(h) => d!(h),
            Hs::S512(h) => d!(h),
            Hs::S3(h) => d!(h),
            Hs::B3(h) => {
                out[..32].copy_from_slice(h.finalize().as_bytes());
                32
            }
            Hs::Crc(h) => {
                out[..4].copy_from_slice(&h.finalize().to_be_bytes());
                4
            }
        }
    }

    pub fn finish(self) -> Vec<u8> {
        let mut b = [0u8; 64];
        let l = self.finish_into(&mut b);
        b[..l].to_vec()
    }
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

pub fn hash_hex(a: Algo, data: &[u8]) -> String {
    let mut h = Hs::new(a);
    h.update(data);
    hex(&h.finish())
}

/// "\n<n в десятичном виде>" в буфер, возвращает длину
pub fn suffix(n: u64, buf: &mut [u8; 24]) -> usize {
    let mut d = [0u8; 20];
    let mut k = 0;
    let mut x = n;
    loop {
        d[k] = b'0' + (x % 10) as u8;
        k += 1;
        x /= 10;
        if x == 0 {
            break;
        }
    }
    buf[0] = b'\n';
    for i in 0..k {
        buf[1 + i] = d[k - 1 - i];
    }
    k + 1
}

pub fn common(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

pub fn fmt_big(x: f64) -> String {
    const U: [(&str, f64); 5] = [("трлн", 1e12), ("млрд", 1e9), ("млн", 1e6), ("тыс", 1e3), ("", 1.0)];
    for (n, v) in U {
        if x >= v {
            return if v == 1.0 { format!("{:.0}", x) } else { format!("{:.2} {}", x / v, n) };
        }
    }
    "0".into()
}

pub fn fmt_dur(s: f64) -> String {
    if !s.is_finite() {
        return "очень долго".into();
    }
    if s < 1.0 {
        "<1 сек".into()
    } else if s < 60.0 {
        format!("{:.0} сек", s)
    } else if s < 3600.0 {
        format!("{:.1} мин", s / 60.0)
    } else if s < 86400.0 {
        format!("{:.1} ч", s / 3600.0)
    } else if s < 86400.0 * 365.0 {
        format!("{:.1} дн", s / 86400.0)
    } else {
        format!("{:.1e} лет", s / 31_557_600.0)
    }
}
