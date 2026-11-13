// GPU-перебор SHA-256 через wgpu (Vulkan/Metal/DX12), compute-шейдер sha256.wgsl
use crate::hashing::*;
use crate::search::{filler, sfx_for, Ctl, IV};
use sha2::compress256;
use sha2::digest::generic_array::GenericArray;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{mpsc, Arc};
use std::time::Instant;

pub struct Gpu {
    pub name: String,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl Gpu {
    pub fn new() -> Result<Gpu, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .ok_or("адаптер не найден (нужен Vulkan-драйвер)")?;
        let info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| e.to_string())?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sha256"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sha256.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Gpu { name: format!("{} ({:?})", info.name, info.backend), device, queue, pipeline })
    }
}

fn block(b: &[u8]) -> &[GenericArray<u8, sha2::digest::consts::U64>] {
    std::slice::from_ref(GenericArray::from_slice(b))
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Первые n раундов SHA-256 (n <= 16, расписание не нужно)
fn rounds(mut s: [u32; 8], w: &[u32; 16], n: usize) -> [u32; 8] {
    for t in 0..n {
        let [a, b, c, d, e, f, g, h] = s;
        let t1 = h
            .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
            .wrapping_add(g ^ (e & (f ^ g)))
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22)).wrapping_add((a & b) | (c & (a | b)));
        s = [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g];
    }
    s
}

fn s0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}
fn s1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

pub fn worker(g: Gpu, base: Arc<String>, val: [u32; 8], tlen: usize, target: String, ctl: Arc<Ctl>, tx: mpsc::Sender<String>) {
    if let Err(e) = run(&g, &base, val, tlen, &target, &ctl, &tx) {
        *ctl.err.lock().unwrap() = Some(e);
        ctl.stop.store(true, Relaxed);
    }
}

fn run(g: &Gpu, base: &Arc<String>, val: [u32; 8], tlen: usize, target: &str, ctl: &Arc<Ctl>, tx: &mpsc::Sender<String>) -> Result<(), String> {
    use wgpu::BufferUsages as U;
    let blen = base.len();
    // Сообщение = base + "\n" + F нулей + hi(8 hex) + lo(8 hex). F подобрано так, что длина = 64*q + 52:
    // все перебираемые байты (hi/lo) попадают в последний блок, а lo ровно на слова W11, W12.
    let f = filler(blen);
    let total = blen + 1 + f + 16;
    let q = total / 64;
    let mut msg = base.as_bytes().to_vec();
    msg.push(b'\n');
    msg.extend(std::iter::repeat(b'0').take(f));
    msg.extend_from_slice(&[b'0'; 16]);
    debug_assert_eq!(total % 64, 52);
    // midstate по всем блокам до последнего
    let mut mid = IV;
    for i in 0..q {
        compress256(&mut mid, block(&msg[i * 64..i * 64 + 64]));
    }
    let tail: Vec<u8> = msg[q * 64..].to_vec(); // 52 байта

    let dev = &g.device;
    let params = dev.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256, usage: U::STORAGE | U::COPY_DST, mapped_at_creation: false });
    let res = dev.create_buffer(&wgpu::BufferDescriptor { label: None, size: 16, usage: U::STORAGE | U::COPY_SRC | U::COPY_DST, mapped_at_creation: false });
    let stage = dev.create_buffer(&wgpu::BufferDescriptor { label: None, size: 16, usage: U::MAP_READ | U::COPY_DST, mapped_at_creation: false });
    let bg = dev.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &g.pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: res.as_entire_binding() },
        ],
    });

    const GROUPS_X: u32 = 4096; // 4096*256 = 2^20 потоков на единицу y
    let mut y: u32 = 4;
    let mut ctr: u64 = 0;
    loop {
        if ctl.stop.load(Relaxed) {
            return Ok(());
        }
        let size: u64 = (GROUPS_X as u64) * 256 * (y as u64);
        if (ctr & 0xFFFF_FFFF) + size > (1u64 << 32) {
            ctr = (ctr | 0xFFFF_FFFF) + 1;
        }
        let hi = (ctr >> 32) as u32;

        // последний блок: 52 байта хвоста, hi в байтах 36..44, lo (44..52) подставит шейдер
        let mut last = [0u8; 64];
        last[..52].copy_from_slice(&tail);
        last[36..44].copy_from_slice(format!("{:08x}", hi).as_bytes());
        last[44..52].fill(0);
        last[52] = 0x80;
        last[56..64].copy_from_slice(&((total as u64) * 8).to_be_bytes());
        let mut w = [0u32; 16];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([last[4 * i], last[4 * i + 1], last[4 * i + 2], last[4 * i + 3]]);
        }
        let s11 = rounds(mid, &w, 11);
        let w16 = s1(w[14]).wrapping_add(w[9]).wrapping_add(s0(w[1])).wrapping_add(w[0]);
        let w17 = s1(w[15]).wrapping_add(w[10]).wrapping_add(s0(w[2])).wrapping_add(w[1]);

        let mut pw = [0u32; 44];
        pw[..8].copy_from_slice(&mid);
        pw[8..16].copy_from_slice(&s11);
        pw[16..24].copy_from_slice(&val);
        pw[24] = tlen as u32;
        pw[25] = ctr as u32;
        pw[26..42].copy_from_slice(&w);
        pw[42] = w16;
        pw[43] = w17;
        let pbytes: Vec<u8> = pw.iter().flat_map(|x| x.to_le_bytes()).collect();
        g.queue.write_buffer(&params, 0, &pbytes);
        g.queue.write_buffer(&res, 0, &[0u8; 16]);

        let t0 = Instant::now();
        let mut enc = dev.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            pass.set_pipeline(&g.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(GROUPS_X, y, 1);
        }
        enc.copy_buffer_to_buffer(&res, 0, &stage, 0, 16);
        g.queue.submit(Some(enc.finish()));
        let (mtx, mrx) = mpsc::channel();
        stage.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = mtx.send(r);
        });
        dev.poll(wgpu::Maintain::Wait);
        mrx.recv().map_err(|e| e.to_string())?.map_err(|e| format!("{:?}", e))?;
        let words: Vec<u32> = {
            let data = stage.slice(..).get_mapped_range();
            data.chunks(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
        };
        stage.unmap();
        let dt = t0.elapsed().as_secs_f64();

        ctl.tries.fetch_add(size, Relaxed);
        ctl.cur.store(ctr, Relaxed);
        if words[2] > 0 {
            ctl.record(words[2] as usize, ((hi as u64) << 32) | words[3] as u64);
        }
        if words[0] == 1 {
            let n = ((hi as u64) << 32) | words[1] as u64;
            let mut text = base.as_bytes().to_vec();
            text.extend_from_slice(&sfx_for(blen, n));
            let h = hash_hex(Algo::Sha256, &text);
            if h.starts_with(target) {
                ctl.stop.store(true, Relaxed);
                let _ = tx.send(String::from_utf8_lossy(&text).into_owned());
                return Ok(());
            }
            return Err(format!("GPU вернул неверный результат (n={:x}, хеш {})", n, h));
        }
        ctr += size;
        // подгоняем размер порции под ~60 мс, чтобы не вешать систему
        let k = (0.06 / dt.max(0.0005)).clamp(0.5, 2.0);
        y = ((y as f64 * k) as u32).clamp(1, 1024);
    }
}
