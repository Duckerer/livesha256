// АВТОГЕНЕРАЦИЯ: tools/gen_shader.py — не править руками.
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

    let w0 = p[26u];
    let w1 = p[27u];
    let w2 = p[28u];
    let w3 = p[29u];
    let w4 = p[30u];
    let w5 = p[31u];
    let w6 = p[32u];
    let w7 = p[33u];
    let w8 = p[34u];
    let w9 = p[35u];
    let w10 = p[36u];
    let w11 = hexpack(lo >> 16u);
    let w12 = hexpack(lo & 0xFFFFu);
    let w13 = p[39u];
    let w14 = p[40u];
    let w15 = p[41u];
    let w16 = p[42u];
    let w17 = p[43u];
    let i0 = p[8u];
    let i1 = p[9u];
    let i2 = p[10u];
    let i3 = p[11u];
    let i4 = p[12u];
    let i5 = p[13u];
    let i6 = p[14u];
    let i7 = p[15u];
    let t11 = i7 + (rotr(i4, 6u) ^ rotr(i4, 11u) ^ rotr(i4, 25u)) + (i6 ^ (i4 & (i5 ^ i6))) + 0x550c7dc3u + w11;
    let e11 = i3 + t11;
    let a11 = t11 + (rotr(i0, 2u) ^ rotr(i0, 13u) ^ rotr(i0, 22u)) + ((i0 & i1) | (i2 & (i0 | i1)));
    let t12 = i6 + (rotr(e11, 6u) ^ rotr(e11, 11u) ^ rotr(e11, 25u)) + (i5 ^ (e11 & (i4 ^ i5))) + 0x72be5d74u + w12;
    let e12 = i2 + t12;
    let a12 = t12 + (rotr(a11, 2u) ^ rotr(a11, 13u) ^ rotr(a11, 22u)) + ((a11 & i0) | (i1 & (a11 | i0)));
    let t13 = i5 + (rotr(e12, 6u) ^ rotr(e12, 11u) ^ rotr(e12, 25u)) + (i4 ^ (e12 & (e11 ^ i4))) + 0x80deb1feu + w13;
    let e13 = i1 + t13;
    let a13 = t13 + (rotr(a12, 2u) ^ rotr(a12, 13u) ^ rotr(a12, 22u)) + ((a12 & a11) | (i0 & (a12 | a11)));
    let t14 = i4 + (rotr(e13, 6u) ^ rotr(e13, 11u) ^ rotr(e13, 25u)) + (e11 ^ (e13 & (e12 ^ e11))) + 0x9bdc06a7u + w14;
    let e14 = i0 + t14;
    let a14 = t14 + (rotr(a13, 2u) ^ rotr(a13, 13u) ^ rotr(a13, 22u)) + ((a13 & a12) | (a11 & (a13 | a12)));
    let t15 = e11 + (rotr(e14, 6u) ^ rotr(e14, 11u) ^ rotr(e14, 25u)) + (e12 ^ (e14 & (e13 ^ e12))) + 0xc19bf174u + w15;
    let e15 = a11 + t15;
    let a15 = t15 + (rotr(a14, 2u) ^ rotr(a14, 13u) ^ rotr(a14, 22u)) + ((a14 & a13) | (a12 & (a14 | a13)));
    let t16 = e12 + (rotr(e15, 6u) ^ rotr(e15, 11u) ^ rotr(e15, 25u)) + (e13 ^ (e15 & (e14 ^ e13))) + 0xe49b69c1u + w16;
    let e16 = a12 + t16;
    let a16 = t16 + (rotr(a15, 2u) ^ rotr(a15, 13u) ^ rotr(a15, 22u)) + ((a15 & a14) | (a13 & (a15 | a14)));
    let t17 = e13 + (rotr(e16, 6u) ^ rotr(e16, 11u) ^ rotr(e16, 25u)) + (e14 ^ (e16 & (e15 ^ e14))) + 0xefbe4786u + w17;
    let e17 = a13 + t17;
    let a17 = t17 + (rotr(a16, 2u) ^ rotr(a16, 13u) ^ rotr(a16, 22u)) + ((a16 & a15) | (a14 & (a16 | a15)));
    let w18 = w2 + (rotr(w3, 7u) ^ rotr(w3, 18u) ^ (w3 >> 3u)) + w11 + (rotr(w16, 17u) ^ rotr(w16, 19u) ^ (w16 >> 10u));
    let t18 = e14 + (rotr(e17, 6u) ^ rotr(e17, 11u) ^ rotr(e17, 25u)) + (e15 ^ (e17 & (e16 ^ e15))) + 0x0fc19dc6u + w18;
    let e18 = a14 + t18;
    let a18 = t18 + (rotr(a17, 2u) ^ rotr(a17, 13u) ^ rotr(a17, 22u)) + ((a17 & a16) | (a15 & (a17 | a16)));
    let w19 = w3 + (rotr(w4, 7u) ^ rotr(w4, 18u) ^ (w4 >> 3u)) + w12 + (rotr(w17, 17u) ^ rotr(w17, 19u) ^ (w17 >> 10u));
    let t19 = e15 + (rotr(e18, 6u) ^ rotr(e18, 11u) ^ rotr(e18, 25u)) + (e16 ^ (e18 & (e17 ^ e16))) + 0x240ca1ccu + w19;
    let e19 = a15 + t19;
    let a19 = t19 + (rotr(a18, 2u) ^ rotr(a18, 13u) ^ rotr(a18, 22u)) + ((a18 & a17) | (a16 & (a18 | a17)));
    let w20 = w4 + (rotr(w5, 7u) ^ rotr(w5, 18u) ^ (w5 >> 3u)) + w13 + (rotr(w18, 17u) ^ rotr(w18, 19u) ^ (w18 >> 10u));
    let t20 = e16 + (rotr(e19, 6u) ^ rotr(e19, 11u) ^ rotr(e19, 25u)) + (e17 ^ (e19 & (e18 ^ e17))) + 0x2de92c6fu + w20;
    let e20 = a16 + t20;
    let a20 = t20 + (rotr(a19, 2u) ^ rotr(a19, 13u) ^ rotr(a19, 22u)) + ((a19 & a18) | (a17 & (a19 | a18)));
    let w21 = w5 + (rotr(w6, 7u) ^ rotr(w6, 18u) ^ (w6 >> 3u)) + w14 + (rotr(w19, 17u) ^ rotr(w19, 19u) ^ (w19 >> 10u));
    let t21 = e17 + (rotr(e20, 6u) ^ rotr(e20, 11u) ^ rotr(e20, 25u)) + (e18 ^ (e20 & (e19 ^ e18))) + 0x4a7484aau + w21;
    let e21 = a17 + t21;
    let a21 = t21 + (rotr(a20, 2u) ^ rotr(a20, 13u) ^ rotr(a20, 22u)) + ((a20 & a19) | (a18 & (a20 | a19)));
    let w22 = w6 + (rotr(w7, 7u) ^ rotr(w7, 18u) ^ (w7 >> 3u)) + w15 + (rotr(w20, 17u) ^ rotr(w20, 19u) ^ (w20 >> 10u));
    let t22 = e18 + (rotr(e21, 6u) ^ rotr(e21, 11u) ^ rotr(e21, 25u)) + (e19 ^ (e21 & (e20 ^ e19))) + 0x5cb0a9dcu + w22;
    let e22 = a18 + t22;
    let a22 = t22 + (rotr(a21, 2u) ^ rotr(a21, 13u) ^ rotr(a21, 22u)) + ((a21 & a20) | (a19 & (a21 | a20)));
    let w23 = w7 + (rotr(w8, 7u) ^ rotr(w8, 18u) ^ (w8 >> 3u)) + w16 + (rotr(w21, 17u) ^ rotr(w21, 19u) ^ (w21 >> 10u));
    let t23 = e19 + (rotr(e22, 6u) ^ rotr(e22, 11u) ^ rotr(e22, 25u)) + (e20 ^ (e22 & (e21 ^ e20))) + 0x76f988dau + w23;
    let e23 = a19 + t23;
    let a23 = t23 + (rotr(a22, 2u) ^ rotr(a22, 13u) ^ rotr(a22, 22u)) + ((a22 & a21) | (a20 & (a22 | a21)));
    let w24 = w8 + (rotr(w9, 7u) ^ rotr(w9, 18u) ^ (w9 >> 3u)) + w17 + (rotr(w22, 17u) ^ rotr(w22, 19u) ^ (w22 >> 10u));
    let t24 = e20 + (rotr(e23, 6u) ^ rotr(e23, 11u) ^ rotr(e23, 25u)) + (e21 ^ (e23 & (e22 ^ e21))) + 0x983e5152u + w24;
    let e24 = a20 + t24;
    let a24 = t24 + (rotr(a23, 2u) ^ rotr(a23, 13u) ^ rotr(a23, 22u)) + ((a23 & a22) | (a21 & (a23 | a22)));
    let w25 = w9 + (rotr(w10, 7u) ^ rotr(w10, 18u) ^ (w10 >> 3u)) + w18 + (rotr(w23, 17u) ^ rotr(w23, 19u) ^ (w23 >> 10u));
    let t25 = e21 + (rotr(e24, 6u) ^ rotr(e24, 11u) ^ rotr(e24, 25u)) + (e22 ^ (e24 & (e23 ^ e22))) + 0xa831c66du + w25;
    let e25 = a21 + t25;
    let a25 = t25 + (rotr(a24, 2u) ^ rotr(a24, 13u) ^ rotr(a24, 22u)) + ((a24 & a23) | (a22 & (a24 | a23)));
    let w26 = w10 + (rotr(w11, 7u) ^ rotr(w11, 18u) ^ (w11 >> 3u)) + w19 + (rotr(w24, 17u) ^ rotr(w24, 19u) ^ (w24 >> 10u));
    let t26 = e22 + (rotr(e25, 6u) ^ rotr(e25, 11u) ^ rotr(e25, 25u)) + (e23 ^ (e25 & (e24 ^ e23))) + 0xb00327c8u + w26;
    let e26 = a22 + t26;
    let a26 = t26 + (rotr(a25, 2u) ^ rotr(a25, 13u) ^ rotr(a25, 22u)) + ((a25 & a24) | (a23 & (a25 | a24)));
    let w27 = w11 + (rotr(w12, 7u) ^ rotr(w12, 18u) ^ (w12 >> 3u)) + w20 + (rotr(w25, 17u) ^ rotr(w25, 19u) ^ (w25 >> 10u));
    let t27 = e23 + (rotr(e26, 6u) ^ rotr(e26, 11u) ^ rotr(e26, 25u)) + (e24 ^ (e26 & (e25 ^ e24))) + 0xbf597fc7u + w27;
    let e27 = a23 + t27;
    let a27 = t27 + (rotr(a26, 2u) ^ rotr(a26, 13u) ^ rotr(a26, 22u)) + ((a26 & a25) | (a24 & (a26 | a25)));
    let w28 = w12 + (rotr(w13, 7u) ^ rotr(w13, 18u) ^ (w13 >> 3u)) + w21 + (rotr(w26, 17u) ^ rotr(w26, 19u) ^ (w26 >> 10u));
    let t28 = e24 + (rotr(e27, 6u) ^ rotr(e27, 11u) ^ rotr(e27, 25u)) + (e25 ^ (e27 & (e26 ^ e25))) + 0xc6e00bf3u + w28;
    let e28 = a24 + t28;
    let a28 = t28 + (rotr(a27, 2u) ^ rotr(a27, 13u) ^ rotr(a27, 22u)) + ((a27 & a26) | (a25 & (a27 | a26)));
    let w29 = w13 + (rotr(w14, 7u) ^ rotr(w14, 18u) ^ (w14 >> 3u)) + w22 + (rotr(w27, 17u) ^ rotr(w27, 19u) ^ (w27 >> 10u));
    let t29 = e25 + (rotr(e28, 6u) ^ rotr(e28, 11u) ^ rotr(e28, 25u)) + (e26 ^ (e28 & (e27 ^ e26))) + 0xd5a79147u + w29;
    let e29 = a25 + t29;
    let a29 = t29 + (rotr(a28, 2u) ^ rotr(a28, 13u) ^ rotr(a28, 22u)) + ((a28 & a27) | (a26 & (a28 | a27)));
    let w30 = w14 + (rotr(w15, 7u) ^ rotr(w15, 18u) ^ (w15 >> 3u)) + w23 + (rotr(w28, 17u) ^ rotr(w28, 19u) ^ (w28 >> 10u));
    let t30 = e26 + (rotr(e29, 6u) ^ rotr(e29, 11u) ^ rotr(e29, 25u)) + (e27 ^ (e29 & (e28 ^ e27))) + 0x06ca6351u + w30;
    let e30 = a26 + t30;
    let a30 = t30 + (rotr(a29, 2u) ^ rotr(a29, 13u) ^ rotr(a29, 22u)) + ((a29 & a28) | (a27 & (a29 | a28)));
    let w31 = w15 + (rotr(w16, 7u) ^ rotr(w16, 18u) ^ (w16 >> 3u)) + w24 + (rotr(w29, 17u) ^ rotr(w29, 19u) ^ (w29 >> 10u));
    let t31 = e27 + (rotr(e30, 6u) ^ rotr(e30, 11u) ^ rotr(e30, 25u)) + (e28 ^ (e30 & (e29 ^ e28))) + 0x14292967u + w31;
    let e31 = a27 + t31;
    let a31 = t31 + (rotr(a30, 2u) ^ rotr(a30, 13u) ^ rotr(a30, 22u)) + ((a30 & a29) | (a28 & (a30 | a29)));
    let w32 = w16 + (rotr(w17, 7u) ^ rotr(w17, 18u) ^ (w17 >> 3u)) + w25 + (rotr(w30, 17u) ^ rotr(w30, 19u) ^ (w30 >> 10u));
    let t32 = e28 + (rotr(e31, 6u) ^ rotr(e31, 11u) ^ rotr(e31, 25u)) + (e29 ^ (e31 & (e30 ^ e29))) + 0x27b70a85u + w32;
    let e32 = a28 + t32;
    let a32 = t32 + (rotr(a31, 2u) ^ rotr(a31, 13u) ^ rotr(a31, 22u)) + ((a31 & a30) | (a29 & (a31 | a30)));
    let w33 = w17 + (rotr(w18, 7u) ^ rotr(w18, 18u) ^ (w18 >> 3u)) + w26 + (rotr(w31, 17u) ^ rotr(w31, 19u) ^ (w31 >> 10u));
    let t33 = e29 + (rotr(e32, 6u) ^ rotr(e32, 11u) ^ rotr(e32, 25u)) + (e30 ^ (e32 & (e31 ^ e30))) + 0x2e1b2138u + w33;
    let e33 = a29 + t33;
    let a33 = t33 + (rotr(a32, 2u) ^ rotr(a32, 13u) ^ rotr(a32, 22u)) + ((a32 & a31) | (a30 & (a32 | a31)));
    let w34 = w18 + (rotr(w19, 7u) ^ rotr(w19, 18u) ^ (w19 >> 3u)) + w27 + (rotr(w32, 17u) ^ rotr(w32, 19u) ^ (w32 >> 10u));
    let t34 = e30 + (rotr(e33, 6u) ^ rotr(e33, 11u) ^ rotr(e33, 25u)) + (e31 ^ (e33 & (e32 ^ e31))) + 0x4d2c6dfcu + w34;
    let e34 = a30 + t34;
    let a34 = t34 + (rotr(a33, 2u) ^ rotr(a33, 13u) ^ rotr(a33, 22u)) + ((a33 & a32) | (a31 & (a33 | a32)));
    let w35 = w19 + (rotr(w20, 7u) ^ rotr(w20, 18u) ^ (w20 >> 3u)) + w28 + (rotr(w33, 17u) ^ rotr(w33, 19u) ^ (w33 >> 10u));
    let t35 = e31 + (rotr(e34, 6u) ^ rotr(e34, 11u) ^ rotr(e34, 25u)) + (e32 ^ (e34 & (e33 ^ e32))) + 0x53380d13u + w35;
    let e35 = a31 + t35;
    let a35 = t35 + (rotr(a34, 2u) ^ rotr(a34, 13u) ^ rotr(a34, 22u)) + ((a34 & a33) | (a32 & (a34 | a33)));
    let w36 = w20 + (rotr(w21, 7u) ^ rotr(w21, 18u) ^ (w21 >> 3u)) + w29 + (rotr(w34, 17u) ^ rotr(w34, 19u) ^ (w34 >> 10u));
    let t36 = e32 + (rotr(e35, 6u) ^ rotr(e35, 11u) ^ rotr(e35, 25u)) + (e33 ^ (e35 & (e34 ^ e33))) + 0x650a7354u + w36;
    let e36 = a32 + t36;
    let a36 = t36 + (rotr(a35, 2u) ^ rotr(a35, 13u) ^ rotr(a35, 22u)) + ((a35 & a34) | (a33 & (a35 | a34)));
    let w37 = w21 + (rotr(w22, 7u) ^ rotr(w22, 18u) ^ (w22 >> 3u)) + w30 + (rotr(w35, 17u) ^ rotr(w35, 19u) ^ (w35 >> 10u));
    let t37 = e33 + (rotr(e36, 6u) ^ rotr(e36, 11u) ^ rotr(e36, 25u)) + (e34 ^ (e36 & (e35 ^ e34))) + 0x766a0abbu + w37;
    let e37 = a33 + t37;
    let a37 = t37 + (rotr(a36, 2u) ^ rotr(a36, 13u) ^ rotr(a36, 22u)) + ((a36 & a35) | (a34 & (a36 | a35)));
    let w38 = w22 + (rotr(w23, 7u) ^ rotr(w23, 18u) ^ (w23 >> 3u)) + w31 + (rotr(w36, 17u) ^ rotr(w36, 19u) ^ (w36 >> 10u));
    let t38 = e34 + (rotr(e37, 6u) ^ rotr(e37, 11u) ^ rotr(e37, 25u)) + (e35 ^ (e37 & (e36 ^ e35))) + 0x81c2c92eu + w38;
    let e38 = a34 + t38;
    let a38 = t38 + (rotr(a37, 2u) ^ rotr(a37, 13u) ^ rotr(a37, 22u)) + ((a37 & a36) | (a35 & (a37 | a36)));
    let w39 = w23 + (rotr(w24, 7u) ^ rotr(w24, 18u) ^ (w24 >> 3u)) + w32 + (rotr(w37, 17u) ^ rotr(w37, 19u) ^ (w37 >> 10u));
    let t39 = e35 + (rotr(e38, 6u) ^ rotr(e38, 11u) ^ rotr(e38, 25u)) + (e36 ^ (e38 & (e37 ^ e36))) + 0x92722c85u + w39;
    let e39 = a35 + t39;
    let a39 = t39 + (rotr(a38, 2u) ^ rotr(a38, 13u) ^ rotr(a38, 22u)) + ((a38 & a37) | (a36 & (a38 | a37)));
    let w40 = w24 + (rotr(w25, 7u) ^ rotr(w25, 18u) ^ (w25 >> 3u)) + w33 + (rotr(w38, 17u) ^ rotr(w38, 19u) ^ (w38 >> 10u));
    let t40 = e36 + (rotr(e39, 6u) ^ rotr(e39, 11u) ^ rotr(e39, 25u)) + (e37 ^ (e39 & (e38 ^ e37))) + 0xa2bfe8a1u + w40;
    let e40 = a36 + t40;
    let a40 = t40 + (rotr(a39, 2u) ^ rotr(a39, 13u) ^ rotr(a39, 22u)) + ((a39 & a38) | (a37 & (a39 | a38)));
    let w41 = w25 + (rotr(w26, 7u) ^ rotr(w26, 18u) ^ (w26 >> 3u)) + w34 + (rotr(w39, 17u) ^ rotr(w39, 19u) ^ (w39 >> 10u));
    let t41 = e37 + (rotr(e40, 6u) ^ rotr(e40, 11u) ^ rotr(e40, 25u)) + (e38 ^ (e40 & (e39 ^ e38))) + 0xa81a664bu + w41;
    let e41 = a37 + t41;
    let a41 = t41 + (rotr(a40, 2u) ^ rotr(a40, 13u) ^ rotr(a40, 22u)) + ((a40 & a39) | (a38 & (a40 | a39)));
    let w42 = w26 + (rotr(w27, 7u) ^ rotr(w27, 18u) ^ (w27 >> 3u)) + w35 + (rotr(w40, 17u) ^ rotr(w40, 19u) ^ (w40 >> 10u));
    let t42 = e38 + (rotr(e41, 6u) ^ rotr(e41, 11u) ^ rotr(e41, 25u)) + (e39 ^ (e41 & (e40 ^ e39))) + 0xc24b8b70u + w42;
    let e42 = a38 + t42;
    let a42 = t42 + (rotr(a41, 2u) ^ rotr(a41, 13u) ^ rotr(a41, 22u)) + ((a41 & a40) | (a39 & (a41 | a40)));
    let w43 = w27 + (rotr(w28, 7u) ^ rotr(w28, 18u) ^ (w28 >> 3u)) + w36 + (rotr(w41, 17u) ^ rotr(w41, 19u) ^ (w41 >> 10u));
    let t43 = e39 + (rotr(e42, 6u) ^ rotr(e42, 11u) ^ rotr(e42, 25u)) + (e40 ^ (e42 & (e41 ^ e40))) + 0xc76c51a3u + w43;
    let e43 = a39 + t43;
    let a43 = t43 + (rotr(a42, 2u) ^ rotr(a42, 13u) ^ rotr(a42, 22u)) + ((a42 & a41) | (a40 & (a42 | a41)));
    let w44 = w28 + (rotr(w29, 7u) ^ rotr(w29, 18u) ^ (w29 >> 3u)) + w37 + (rotr(w42, 17u) ^ rotr(w42, 19u) ^ (w42 >> 10u));
    let t44 = e40 + (rotr(e43, 6u) ^ rotr(e43, 11u) ^ rotr(e43, 25u)) + (e41 ^ (e43 & (e42 ^ e41))) + 0xd192e819u + w44;
    let e44 = a40 + t44;
    let a44 = t44 + (rotr(a43, 2u) ^ rotr(a43, 13u) ^ rotr(a43, 22u)) + ((a43 & a42) | (a41 & (a43 | a42)));
    let w45 = w29 + (rotr(w30, 7u) ^ rotr(w30, 18u) ^ (w30 >> 3u)) + w38 + (rotr(w43, 17u) ^ rotr(w43, 19u) ^ (w43 >> 10u));
    let t45 = e41 + (rotr(e44, 6u) ^ rotr(e44, 11u) ^ rotr(e44, 25u)) + (e42 ^ (e44 & (e43 ^ e42))) + 0xd6990624u + w45;
    let e45 = a41 + t45;
    let a45 = t45 + (rotr(a44, 2u) ^ rotr(a44, 13u) ^ rotr(a44, 22u)) + ((a44 & a43) | (a42 & (a44 | a43)));
    let w46 = w30 + (rotr(w31, 7u) ^ rotr(w31, 18u) ^ (w31 >> 3u)) + w39 + (rotr(w44, 17u) ^ rotr(w44, 19u) ^ (w44 >> 10u));
    let t46 = e42 + (rotr(e45, 6u) ^ rotr(e45, 11u) ^ rotr(e45, 25u)) + (e43 ^ (e45 & (e44 ^ e43))) + 0xf40e3585u + w46;
    let e46 = a42 + t46;
    let a46 = t46 + (rotr(a45, 2u) ^ rotr(a45, 13u) ^ rotr(a45, 22u)) + ((a45 & a44) | (a43 & (a45 | a44)));
    let w47 = w31 + (rotr(w32, 7u) ^ rotr(w32, 18u) ^ (w32 >> 3u)) + w40 + (rotr(w45, 17u) ^ rotr(w45, 19u) ^ (w45 >> 10u));
    let t47 = e43 + (rotr(e46, 6u) ^ rotr(e46, 11u) ^ rotr(e46, 25u)) + (e44 ^ (e46 & (e45 ^ e44))) + 0x106aa070u + w47;
    let e47 = a43 + t47;
    let a47 = t47 + (rotr(a46, 2u) ^ rotr(a46, 13u) ^ rotr(a46, 22u)) + ((a46 & a45) | (a44 & (a46 | a45)));
    let w48 = w32 + (rotr(w33, 7u) ^ rotr(w33, 18u) ^ (w33 >> 3u)) + w41 + (rotr(w46, 17u) ^ rotr(w46, 19u) ^ (w46 >> 10u));
    let t48 = e44 + (rotr(e47, 6u) ^ rotr(e47, 11u) ^ rotr(e47, 25u)) + (e45 ^ (e47 & (e46 ^ e45))) + 0x19a4c116u + w48;
    let e48 = a44 + t48;
    let a48 = t48 + (rotr(a47, 2u) ^ rotr(a47, 13u) ^ rotr(a47, 22u)) + ((a47 & a46) | (a45 & (a47 | a46)));
    let w49 = w33 + (rotr(w34, 7u) ^ rotr(w34, 18u) ^ (w34 >> 3u)) + w42 + (rotr(w47, 17u) ^ rotr(w47, 19u) ^ (w47 >> 10u));
    let t49 = e45 + (rotr(e48, 6u) ^ rotr(e48, 11u) ^ rotr(e48, 25u)) + (e46 ^ (e48 & (e47 ^ e46))) + 0x1e376c08u + w49;
    let e49 = a45 + t49;
    let a49 = t49 + (rotr(a48, 2u) ^ rotr(a48, 13u) ^ rotr(a48, 22u)) + ((a48 & a47) | (a46 & (a48 | a47)));
    let w50 = w34 + (rotr(w35, 7u) ^ rotr(w35, 18u) ^ (w35 >> 3u)) + w43 + (rotr(w48, 17u) ^ rotr(w48, 19u) ^ (w48 >> 10u));
    let t50 = e46 + (rotr(e49, 6u) ^ rotr(e49, 11u) ^ rotr(e49, 25u)) + (e47 ^ (e49 & (e48 ^ e47))) + 0x2748774cu + w50;
    let e50 = a46 + t50;
    let a50 = t50 + (rotr(a49, 2u) ^ rotr(a49, 13u) ^ rotr(a49, 22u)) + ((a49 & a48) | (a47 & (a49 | a48)));
    let w51 = w35 + (rotr(w36, 7u) ^ rotr(w36, 18u) ^ (w36 >> 3u)) + w44 + (rotr(w49, 17u) ^ rotr(w49, 19u) ^ (w49 >> 10u));
    let t51 = e47 + (rotr(e50, 6u) ^ rotr(e50, 11u) ^ rotr(e50, 25u)) + (e48 ^ (e50 & (e49 ^ e48))) + 0x34b0bcb5u + w51;
    let e51 = a47 + t51;
    let a51 = t51 + (rotr(a50, 2u) ^ rotr(a50, 13u) ^ rotr(a50, 22u)) + ((a50 & a49) | (a48 & (a50 | a49)));
    let w52 = w36 + (rotr(w37, 7u) ^ rotr(w37, 18u) ^ (w37 >> 3u)) + w45 + (rotr(w50, 17u) ^ rotr(w50, 19u) ^ (w50 >> 10u));
    let t52 = e48 + (rotr(e51, 6u) ^ rotr(e51, 11u) ^ rotr(e51, 25u)) + (e49 ^ (e51 & (e50 ^ e49))) + 0x391c0cb3u + w52;
    let e52 = a48 + t52;
    let a52 = t52 + (rotr(a51, 2u) ^ rotr(a51, 13u) ^ rotr(a51, 22u)) + ((a51 & a50) | (a49 & (a51 | a50)));
    let w53 = w37 + (rotr(w38, 7u) ^ rotr(w38, 18u) ^ (w38 >> 3u)) + w46 + (rotr(w51, 17u) ^ rotr(w51, 19u) ^ (w51 >> 10u));
    let t53 = e49 + (rotr(e52, 6u) ^ rotr(e52, 11u) ^ rotr(e52, 25u)) + (e50 ^ (e52 & (e51 ^ e50))) + 0x4ed8aa4au + w53;
    let e53 = a49 + t53;
    let a53 = t53 + (rotr(a52, 2u) ^ rotr(a52, 13u) ^ rotr(a52, 22u)) + ((a52 & a51) | (a50 & (a52 | a51)));
    let w54 = w38 + (rotr(w39, 7u) ^ rotr(w39, 18u) ^ (w39 >> 3u)) + w47 + (rotr(w52, 17u) ^ rotr(w52, 19u) ^ (w52 >> 10u));
    let t54 = e50 + (rotr(e53, 6u) ^ rotr(e53, 11u) ^ rotr(e53, 25u)) + (e51 ^ (e53 & (e52 ^ e51))) + 0x5b9cca4fu + w54;
    let e54 = a50 + t54;
    let a54 = t54 + (rotr(a53, 2u) ^ rotr(a53, 13u) ^ rotr(a53, 22u)) + ((a53 & a52) | (a51 & (a53 | a52)));
    let w55 = w39 + (rotr(w40, 7u) ^ rotr(w40, 18u) ^ (w40 >> 3u)) + w48 + (rotr(w53, 17u) ^ rotr(w53, 19u) ^ (w53 >> 10u));
    let t55 = e51 + (rotr(e54, 6u) ^ rotr(e54, 11u) ^ rotr(e54, 25u)) + (e52 ^ (e54 & (e53 ^ e52))) + 0x682e6ff3u + w55;
    let e55 = a51 + t55;
    let a55 = t55 + (rotr(a54, 2u) ^ rotr(a54, 13u) ^ rotr(a54, 22u)) + ((a54 & a53) | (a52 & (a54 | a53)));
    let w56 = w40 + (rotr(w41, 7u) ^ rotr(w41, 18u) ^ (w41 >> 3u)) + w49 + (rotr(w54, 17u) ^ rotr(w54, 19u) ^ (w54 >> 10u));
    let t56 = e52 + (rotr(e55, 6u) ^ rotr(e55, 11u) ^ rotr(e55, 25u)) + (e53 ^ (e55 & (e54 ^ e53))) + 0x748f82eeu + w56;
    let e56 = a52 + t56;
    let a56 = t56 + (rotr(a55, 2u) ^ rotr(a55, 13u) ^ rotr(a55, 22u)) + ((a55 & a54) | (a53 & (a55 | a54)));
    let w57 = w41 + (rotr(w42, 7u) ^ rotr(w42, 18u) ^ (w42 >> 3u)) + w50 + (rotr(w55, 17u) ^ rotr(w55, 19u) ^ (w55 >> 10u));
    let t57 = e53 + (rotr(e56, 6u) ^ rotr(e56, 11u) ^ rotr(e56, 25u)) + (e54 ^ (e56 & (e55 ^ e54))) + 0x78a5636fu + w57;
    let e57 = a53 + t57;
    let a57 = t57 + (rotr(a56, 2u) ^ rotr(a56, 13u) ^ rotr(a56, 22u)) + ((a56 & a55) | (a54 & (a56 | a55)));
    let w58 = w42 + (rotr(w43, 7u) ^ rotr(w43, 18u) ^ (w43 >> 3u)) + w51 + (rotr(w56, 17u) ^ rotr(w56, 19u) ^ (w56 >> 10u));
    let t58 = e54 + (rotr(e57, 6u) ^ rotr(e57, 11u) ^ rotr(e57, 25u)) + (e55 ^ (e57 & (e56 ^ e55))) + 0x84c87814u + w58;
    let e58 = a54 + t58;
    let a58 = t58 + (rotr(a57, 2u) ^ rotr(a57, 13u) ^ rotr(a57, 22u)) + ((a57 & a56) | (a55 & (a57 | a56)));
    let w59 = w43 + (rotr(w44, 7u) ^ rotr(w44, 18u) ^ (w44 >> 3u)) + w52 + (rotr(w57, 17u) ^ rotr(w57, 19u) ^ (w57 >> 10u));
    let t59 = e55 + (rotr(e58, 6u) ^ rotr(e58, 11u) ^ rotr(e58, 25u)) + (e56 ^ (e58 & (e57 ^ e56))) + 0x8cc70208u + w59;
    let e59 = a55 + t59;
    let a59 = t59 + (rotr(a58, 2u) ^ rotr(a58, 13u) ^ rotr(a58, 22u)) + ((a58 & a57) | (a56 & (a58 | a57)));
    let w60 = w44 + (rotr(w45, 7u) ^ rotr(w45, 18u) ^ (w45 >> 3u)) + w53 + (rotr(w58, 17u) ^ rotr(w58, 19u) ^ (w58 >> 10u));
    let t60 = e56 + (rotr(e59, 6u) ^ rotr(e59, 11u) ^ rotr(e59, 25u)) + (e57 ^ (e59 & (e58 ^ e57))) + 0x90befffau + w60;
    let e60 = a56 + t60;
    let a60 = t60 + (rotr(a59, 2u) ^ rotr(a59, 13u) ^ rotr(a59, 22u)) + ((a59 & a58) | (a57 & (a59 | a58)));
    let w61 = w45 + (rotr(w46, 7u) ^ rotr(w46, 18u) ^ (w46 >> 3u)) + w54 + (rotr(w59, 17u) ^ rotr(w59, 19u) ^ (w59 >> 10u));
    let t61 = e57 + (rotr(e60, 6u) ^ rotr(e60, 11u) ^ rotr(e60, 25u)) + (e58 ^ (e60 & (e59 ^ e58))) + 0xa4506cebu + w61;
    let e61 = a57 + t61;
    let a61 = t61 + (rotr(a60, 2u) ^ rotr(a60, 13u) ^ rotr(a60, 22u)) + ((a60 & a59) | (a58 & (a60 | a59)));
    let w62 = w46 + (rotr(w47, 7u) ^ rotr(w47, 18u) ^ (w47 >> 3u)) + w55 + (rotr(w60, 17u) ^ rotr(w60, 19u) ^ (w60 >> 10u));
    let t62 = e58 + (rotr(e61, 6u) ^ rotr(e61, 11u) ^ rotr(e61, 25u)) + (e59 ^ (e61 & (e60 ^ e59))) + 0xbef9a3f7u + w62;
    let e62 = a58 + t62;
    let a62 = t62 + (rotr(a61, 2u) ^ rotr(a61, 13u) ^ rotr(a61, 22u)) + ((a61 & a60) | (a59 & (a61 | a60)));
    let w63 = w47 + (rotr(w48, 7u) ^ rotr(w48, 18u) ^ (w48 >> 3u)) + w56 + (rotr(w61, 17u) ^ rotr(w61, 19u) ^ (w61 >> 10u));
    let t63 = e59 + (rotr(e62, 6u) ^ rotr(e62, 11u) ^ rotr(e62, 25u)) + (e60 ^ (e62 & (e61 ^ e60))) + 0xc67178f2u + w63;
    let e63 = a59 + t63;
    let a63 = t63 + (rotr(a62, 2u) ^ rotr(a62, 13u) ^ rotr(a62, 22u)) + ((a62 & a61) | (a60 & (a62 | a61)));
    let d0 = p[0u] + a63;
    let d1 = p[1u] + a62;
    let d2 = p[2u] + a61;
    let d3 = p[3u] + a60;
    let d4 = p[4u] + e63;
    let d5 = p[5u] + e62;
    let d6 = p[6u] + e61;
    let d7 = p[7u] + e60;
    var mm = countLeadingZeros(d0 ^ p[16u]) / 4u;
    if (mm == 8u) {
        mm += countLeadingZeros(d1 ^ p[17u]) / 4u;
        if (mm == 16u) {
            mm += countLeadingZeros(d2 ^ p[18u]) / 4u;
            if (mm == 24u) {
                mm += countLeadingZeros(d3 ^ p[19u]) / 4u;
                if (mm == 32u) {
                    mm += countLeadingZeros(d4 ^ p[20u]) / 4u;
                    if (mm == 40u) {
                        mm += countLeadingZeros(d5 ^ p[21u]) / 4u;
                        if (mm == 48u) {
                            mm += countLeadingZeros(d6 ^ p[22u]) / 4u;
                            if (mm == 56u) {
                                mm += countLeadingZeros(d7 ^ p[23u]) / 4u;
                            }
                        }
                    }
                }
            }
        }
    }
    mm = min(mm, p[24u]);
    if (mm > atomicLoad(&res[2])) {
        atomicMax(&res[2], mm);
        atomicStore(&res[3], lo);
    }
    if (mm >= p[24u]) {
        atomicStore(&res[1], lo);
        atomicStore(&res[0], 1u);
    }
}
