//! Pure-Rust Widevine L3 cryptographic primitives (RSA-2048 PSS/OAEP, SHA-1, AES-128-CMAC, Protobuf).

use crate::error::{CalyxError, CalyxResult};
use aes::Aes128;
use aes::cipher::{BlockDecryptMut, BlockEncrypt, KeyInit, KeyIvInit, generic_array::GenericArray};

type Aes128CbcDec = cbc::Decryptor<Aes128>;

pub fn parse_rsa2048_der(der: &[u8]) -> CalyxResult<([u8; 256], [u8; 256])> {
    if der.len() < 520 || der[0] != 0x30 {
        return Err(CalyxError::Drm("Invalid RSA DER header".to_string()));
    }
    let pos = if der[1] & 0x80 != 0 {
        2 + (der[1] & 0x7f) as usize
    } else {
        2
    };
    let (_ver, p1) = read_asn1_int(der, pos)?;
    let (n_slice, p2) = read_asn1_int(der, p1)?;
    let (_e, p3) = read_asn1_int(der, p2)?;
    let (d_slice, _) = read_asn1_int(der, p3)?;
    Ok((to_fixed_256(n_slice), to_fixed_256(d_slice)))
}

fn read_asn1_int(buf: &[u8], mut pos: usize) -> CalyxResult<(&[u8], usize)> {
    if pos + 2 > buf.len() || buf[pos] != 0x02 {
        return Err(CalyxError::Drm("ASN.1 INTEGER expected".to_string()));
    }
    pos += 1;
    let mut len = buf[pos] as usize;
    pos += 1;
    if len & 0x80 != 0 {
        let nbytes = len & 0x7f;
        if nbytes > 4 || pos + nbytes > buf.len() {
            return Err(CalyxError::Drm("Invalid ASN.1 length".to_string()));
        }
        len = 0;
        for &b in &buf[pos..pos + nbytes] {
            len = (len << 8) | (b as usize);
        }
        pos += nbytes;
    }
    if pos + len > buf.len() {
        return Err(CalyxError::Drm("ASN.1 INTEGER truncated".to_string()));
    }
    let mut slice = &buf[pos..pos + len];
    while slice.len() > 1 && slice[0] == 0 {
        slice = &slice[1..];
    }
    Ok((slice, pos + len))
}

fn to_fixed_256(src: &[u8]) -> [u8; 256] {
    let mut out = [0u8; 256];
    let take = src.len().min(256);
    out[256 - take..].copy_from_slice(&src[src.len() - take..]);
    out
}

pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, &v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn mgf1_sha1(seed: &[u8], len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len + 20);
    let mut counter = 0u32;
    while out.len() < len {
        let mut buf = Vec::with_capacity(seed.len() + 4);
        buf.extend_from_slice(seed);
        buf.extend_from_slice(&counter.to_be_bytes());
        out.extend_from_slice(&sha1(&buf));
        counter += 1;
    }
    out.truncate(len);
    out
}

pub fn rsa2048_modpow(base_be: &[u8; 256], exp_be: &[u8; 256], mod_be: &[u8; 256]) -> [u8; 256] {
    let mut result = [0u64; 32];
    result[0] = 1;
    let base = bytes_to_limbs(base_be);
    let modulus = bytes_to_limbs(mod_be);
    for &byte in exp_be.iter() {
        for bit in (0..8).rev() {
            result = mul_mod(&result, &result, &modulus);
            if (byte >> bit) & 1 == 1 {
                result = mul_mod(&result, &base, &modulus);
            }
        }
    }
    limbs_to_bytes(&result)
}

fn bytes_to_limbs(be: &[u8; 256]) -> [u64; 32] {
    let mut limbs = [0u64; 32];
    for (i, limb) in limbs.iter_mut().enumerate() {
        let idx = 256 - (i + 1) * 8;
        *limb = u64::from_be_bytes(be[idx..idx + 8].try_into().unwrap_or([0; 8]));
    }
    limbs
}

fn limbs_to_bytes(limbs: &[u64; 32]) -> [u8; 256] {
    let mut out = [0u8; 256];
    for (i, limb) in limbs.iter().enumerate() {
        let idx = 256 - (i + 1) * 8;
        out[idx..idx + 8].copy_from_slice(&limb.to_be_bytes());
    }
    out
}

fn mul_mod(a: &[u64; 32], b: &[u64; 32], m: &[u64; 32]) -> [u64; 32] {
    let mut prod = [0u64; 64];
    for i in 0..32 {
        let mut carry = 0u128;
        for j in 0..32 {
            let cur = (prod[i + j] as u128) + (a[i] as u128) * (b[j] as u128) + carry;
            prod[i + j] = cur as u64;
            carry = cur >> 64;
        }
        prod[i + 32] = carry as u64;
    }
    let mut rem = [0u64; 32];
    for bit_idx in (0..4096).rev() {
        let mut carry = (prod[bit_idx / 64] >> (bit_idx % 64)) & 1;
        for limb in &mut rem {
            let next_c = *limb >> 63;
            *limb = (*limb << 1) | carry;
            carry = next_c;
        }
        if carry == 1 || cmp_limbs(&rem, m) >= 0 {
            sub_assign_limbs(&mut rem, m);
        }
    }
    rem
}

fn cmp_limbs(a: &[u64; 32], b: &[u64; 32]) -> i8 {
    for i in (0..32).rev() {
        if a[i] > b[i] {
            return 1;
        } else if a[i] < b[i] {
            return -1;
        }
    }
    0
}

fn sub_assign_limbs(a: &mut [u64; 32], b: &[u64; 32]) {
    let mut borrow = 0u64;
    for i in 0..32 {
        let (d1, b1) = a[i].overflowing_sub(b[i]);
        let (d2, b2) = d1.overflowing_sub(borrow);
        a[i] = d2;
        borrow = (b1 as u64) + (b2 as u64);
    }
}

pub fn rsa_pss_sha1_sign(msg: &[u8], d: &[u8; 256], n: &[u8; 256]) -> [u8; 256] {
    let m_hash = sha1(msg);
    let salt = sha1(&[&m_hash[..], b"calyx-wv-salt"].concat());
    let mut m_prime = vec![0u8; 8];
    m_prime.extend_from_slice(&m_hash);
    m_prime.extend_from_slice(&salt);
    let h = sha1(&m_prime);
    let mut db = vec![0u8; 256 - 20 - 20 - 2];
    db.push(0x01);
    db.extend_from_slice(&salt);
    let db_mask = mgf1_sha1(&h, db.len());
    for (x, m) in db.iter_mut().zip(db_mask.iter()) {
        *x ^= *m;
    }
    db[0] &= 0x7f;
    let mut em = [0u8; 256];
    em[..235].copy_from_slice(&db);
    em[235..255].copy_from_slice(&h);
    em[255] = 0xbc;
    rsa2048_modpow(&em, d, n)
}

pub fn rsa_oaep_sha1_decrypt(ct: &[u8; 256], d: &[u8; 256], n: &[u8; 256]) -> CalyxResult<Vec<u8>> {
    let em = rsa2048_modpow(ct, d, n);
    let mut seed = [0u8; 20];
    seed.copy_from_slice(&em[1..21]);
    let mut db = em[21..256].to_vec();
    let seed_mask = mgf1_sha1(&db, 20);
    for (s, m) in seed.iter_mut().zip(seed_mask.iter()) {
        *s ^= *m;
    }
    let db_mask = mgf1_sha1(&seed, db.len());
    for (x, m) in db.iter_mut().zip(db_mask.iter()) {
        *x ^= *m;
    }
    let sep = db[20..]
        .iter()
        .position(|&b| b == 0x01)
        .ok_or_else(|| CalyxError::Drm("RSA-OAEP padding separator 0x01 not found".to_string()))?;
    Ok(db[20 + sep + 1..].to_vec())
}

pub fn aes128_cmac(key: &[u8; 16], data: &[u8]) -> [u8; 16] {
    let cipher = Aes128::new(key.into());
    let mut l = [0u8; 16];
    cipher.encrypt_block(GenericArray::from_mut_slice(&mut l));
    let k1 = dbl_block(l);
    let k2 = dbl_block(k1);
    let n_blocks = if data.is_empty() {
        1
    } else {
        data.len().div_ceil(16)
    };
    let complete = !data.is_empty() && data.len().is_multiple_of(16);
    let mut x = [0u8; 16];
    for i in 0..n_blocks {
        let mut m_block = [0u8; 16];
        let start = i * 16;
        if i + 1 < n_blocks {
            m_block.copy_from_slice(&data[start..start + 16]);
        } else if complete {
            m_block.copy_from_slice(&data[start..start + 16]);
            for j in 0..16 {
                m_block[j] ^= k1[j];
            }
        } else {
            let rem = data.len().saturating_sub(start);
            if rem > 0 {
                m_block[..rem].copy_from_slice(&data[start..]);
            }
            m_block[rem] = 0x80;
            for j in 0..16 {
                m_block[j] ^= k2[j];
            }
        }
        for j in 0..16 {
            x[j] ^= m_block[j];
        }
        cipher.encrypt_block(GenericArray::from_mut_slice(&mut x));
    }
    x
}

fn dbl_block(b: [u8; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut carry = 0u8;
    for i in (0..16).rev() {
        let next = b[i] >> 7;
        out[i] = (b[i] << 1) | carry;
        carry = next;
    }
    if (b[0] & 0x80) != 0 {
        out[15] ^= 0x87;
    }
    out
}

pub fn decrypt_aes128_cbc_key(
    enc_key: &[u8; 16],
    iv: &[u8; 16],
    ct: &[u8],
) -> CalyxResult<[u8; 16]> {
    if ct.len() < 16 {
        return Err(CalyxError::Drm(
            "Encrypted content_key < 16 bytes".to_string(),
        ));
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&ct[..16]);
    let mut dec = Aes128CbcDec::new(enc_key.into(), iv.into());
    dec.decrypt_block_mut(GenericArray::from_mut_slice(&mut buf));
    Ok(buf)
}
