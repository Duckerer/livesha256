#!/usr/bin/env python3
# Генератор раскрученного WGSL-шейдера SHA-256 (src/sha256.wgsl).
# Раскладка последнего блока: перебираемые 8 hex-символов лежат в байтах 44..51 (слова W11, W12),
# поэтому раунды 0..10 и слова расписания W16, W17 считаются на хосте.
K = [
0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2]
START = 11
o = []
o.append("""// АВТОГЕНЕРАЦИЯ: tools/gen_shader.py — не править руками.
// params: [0..8] midstate, [8..16] состояние после раундов 0..10, [16..24] целевой префикс,
// [24] длина префикса (нибблы), [25] base_lo, [26..42] W0..W15, [42] W16, [43] W17
@group(0) @binding(0) var<storage, read> p: array<u32>;
@group(0) @binding(1) var<storage, read_write> res: array<atomic<u32>, 4>;

fn rotr(x: u32, n: u32) -> u32 { return (x >> n) | (x << (32u - n)); }

// 16 бит -> 4 ASCII hex-символа в одном слове (big-endian)
fn hexpack(h: u32) -> u32 {
    let x = ((h & 0xF000u) << 12u) | ((h & 0x0F00u) << 8u) | ((h & 0x00F0u) << 4u) | (h & 0x000Fu);
    return x + 0x30303030u + (((x + 0x06060606u) >> 4u) & 0x01010101u) * 39u;
}

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) wid: vec3<u32>, @builtin(num_workgroups) nwg: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let gid = (wid.y * nwg.x + wid.x) * 256u + li;
    let lo = p[25] + gid;
""")
for i in range(16):
    if i == 11: o.append("    let w11 = hexpack(lo >> 16u);")
    elif i == 12: o.append("    let w12 = hexpack(lo & 0xFFFFu);")
    else: o.append(f"    let w{i} = p[{26+i}u];")
o.append("    let w16 = p[42u];\n    let w17 = p[43u];")
regs = [f"i{j}" for j in range(8)]
for j in range(8): o.append(f"    let i{j} = p[{8+j}u];")
for t in range(START, 64):
    if t >= 18:
        o.append(f"    let w{t} = w{t-16} + (rotr(w{t-15}, 7u) ^ rotr(w{t-15}, 18u) ^ (w{t-15} >> 3u)) + w{t-7} + (rotr(w{t-2}, 17u) ^ rotr(w{t-2}, 19u) ^ (w{t-2} >> 10u));")
    a,b,c,d,e,f,g,h = regs
    o.append(f"    let t{t} = {h} + (rotr({e}, 6u) ^ rotr({e}, 11u) ^ rotr({e}, 25u)) + ({g} ^ ({e} & ({f} ^ {g}))) + 0x{K[t]:08x}u + w{t};")
    o.append(f"    let e{t} = {d} + t{t};")
    o.append(f"    let a{t} = t{t} + (rotr({a}, 2u) ^ rotr({a}, 13u) ^ rotr({a}, 22u)) + (({a} & {b}) | ({c} & ({a} | {b})));")
    regs = [f"a{t}", a, b, c, f"e{t}", e, f, g]
for j in range(8): o.append(f"    let d{j} = p[{j}u] + {regs[j]};")
o.append("""    var mm = countLeadingZeros(d0 ^ p[16u]) / 4u;""")
ind = "    "
for j in range(1, 8):
    o.append(f"{ind}if (mm == {8*j}u) {{")
    ind += "    "
    o.append(f"{ind}mm += countLeadingZeros(d{j} ^ p[{16+j}u]) / 4u;")
for j in range(7): 
    ind = ind[:-4]; o.append(f"{ind}}}")
o.append("""    mm = min(mm, p[24u]);
    if (mm > atomicLoad(&res[2])) {
        atomicMax(&res[2], mm);
        atomicStore(&res[3], lo);
    }
    if (mm >= p[24u]) {
        atomicStore(&res[1], lo);
        atomicStore(&res[0], 1u);
    }
}""")
open("src/sha256.wgsl", "w").write("\n".join(o) + "\n")
print("ok", len(o), "строк")
