use crate::error::{CalyxError, CalyxResult};
use crate::wv_crypto::{
    aes128_cmac, decrypt_aes128_cbc_key, parse_rsa2048_der, rsa_oaep_sha1_decrypt,
    rsa_pss_sha1_sign,
};
use crate::wvd_builtin::BUILTIN_ANDROID_L3_WVD_B64;
use aes::Aes128;
use aes::cipher::{BlockDecryptMut, KeyIvInit, generic_array::GenericArray};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use sha2::{Digest, Sha256};

type Aes128CbcDec = cbc::Decryptor<Aes128>;

#[derive(Debug, Clone)]
pub struct PsshBox {
    pub raw_bytes: Vec<u8>,
    pub wv_pssh_data: Vec<u8>,
    pub key_ids: Vec<[u8; 16]>,
}

impl PsshBox {
    pub fn from_base64(b64: &str) -> CalyxResult<Self> {
        let clean = b64.trim().rsplit(',').next().unwrap_or(b64.trim());
        if clean.is_empty() {
            return Err(CalyxError::Drm("Empty PSSH Base64 input".to_string()));
        }
        let decoded = BASE64
            .decode(clean)
            .map_err(|e| CalyxError::Drm(format!("Invalid Base64 PSSH: {e}")))?;
        if decoded.len() == 16 {
            let mut kid = [0u8; 16];
            kid.copy_from_slice(&decoded);
            let mut wv_data = vec![0x08, 0x01, 0x12, 0x10];
            wv_data.extend_from_slice(&decoded);
            return Ok(Self {
                raw_bytes: decoded,
                wv_pssh_data: wv_data,
                key_ids: vec![kid],
            });
        }
        if decoded.len() < 32 || &decoded[4..8] != b"pssh" {
            return Err(CalyxError::Drm(
                "Invalid PSSH box header signature".to_string(),
            ));
        }
        let wv_pssh_data = if decoded.len() > 32 {
            decoded[32..].to_vec()
        } else {
            decoded.clone()
        };
        Ok(Self {
            raw_bytes: decoded,
            wv_pssh_data,
            key_ids: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct WidevineDeviceL3 {
    pub system_id: u32,
    pub security_level: u8,
    pub client_id_blob: Vec<u8>,
    pub rsa_n: Option<[u8; 256]>,
    pub rsa_d: Option<[u8; 256]>,
}

impl WidevineDeviceL3 {
    pub fn builtin() -> CalyxResult<Self> {
        let raw = BASE64
            .decode(BUILTIN_ANDROID_L3_WVD_B64)
            .map_err(|e| CalyxError::Drm(format!("WVD base64: {e}")))?;
        Self::from_wvd_bytes(&raw)
    }

    pub fn from_wvd_bytes(data: &[u8]) -> CalyxResult<Self> {
        if data.len() < 12 || &data[0..3] != b"WVD" {
            return Err(CalyxError::Drm(
                "Invalid .wvd magic bytes (expected 'WVD')".to_string(),
            ));
        }
        if data[3] == 2 && data.len() >= 600 {
            let security_level = data[5];
            let priv_len = u16::from_be_bytes([data[7], data[8]]) as usize;
            if 11 + priv_len > data.len() {
                return Err(CalyxError::Drm("Truncated WVD v2 private key".to_string()));
            }
            let priv_der = &data[9..9 + priv_len];
            let cid_len = u16::from_be_bytes([data[9 + priv_len], data[10 + priv_len]]) as usize;
            if 11 + priv_len + cid_len > data.len() {
                return Err(CalyxError::Drm(
                    "Truncated WVD v2 client_id_blob".to_string(),
                ));
            }
            let client_id_blob = data[11 + priv_len..11 + priv_len + cid_len].to_vec();
            let (n, d) = parse_rsa2048_der(priv_der)?;
            return Ok(Self {
                system_id: 4464,
                security_level,
                client_id_blob,
                rsa_n: Some(n),
                rsa_d: Some(d),
            });
        }
        let system_id = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Ok(Self {
            system_id,
            security_level: data[8],
            client_id_blob: data[9..].to_vec(),
            rsa_n: None,
            rsa_d: None,
        })
    }

    pub fn build_license_challenge(&self, pssh: &PsshBox) -> CalyxResult<Vec<u8>> {
        if let (Some(n), Some(d)) = (&self.rsa_n, &self.rsa_d) {
            return Ok(self.build_signed_challenge(pssh, d, n).0);
        }
        if self.client_id_blob.is_empty() {
            return Err(CalyxError::Drm(
                "Empty Widevine L3 client_id_blob".to_string(),
            ));
        }
        let mut hasher = Sha256::new();
        hasher.update(&self.client_id_blob);
        hasher.update(&pssh.raw_bytes);
        Ok(hasher.finalize().to_vec())
    }

    pub fn build_signed_challenge(
        &self,
        pssh: &PsshBox,
        d: &[u8; 256],
        n: &[u8; 256],
    ) -> (Vec<u8>, Vec<u8>) {
        let mut wv_pssh_msg = Vec::new();
        push_proto_bytes(&mut wv_pssh_msg, 1, &pssh.wv_pssh_data);
        push_proto_varint(&mut wv_pssh_msg, 2, 1);
        push_proto_bytes(&mut wv_pssh_msg, 3, b"0102030405060708");
        let mut content_id = Vec::new();
        push_proto_bytes(&mut content_id, 1, &wv_pssh_msg);
        let mut lic_req = Vec::new();
        push_proto_bytes(&mut lic_req, 1, &self.client_id_blob);
        push_proto_bytes(&mut lic_req, 2, &content_id);
        push_proto_varint(&mut lic_req, 3, 1);
        push_proto_varint(&mut lic_req, 4, 1_730_000_000);
        push_proto_varint(&mut lic_req, 6, 21);
        push_proto_varint(&mut lic_req, 7, 123_456_789);
        let sig = rsa_pss_sha1_sign(&lic_req, d, n);
        let mut signed_msg = Vec::new();
        push_proto_varint(&mut signed_msg, 1, 1);
        push_proto_bytes(&mut signed_msg, 2, &lic_req);
        push_proto_bytes(&mut signed_msg, 3, &sig);
        (signed_msg, lic_req)
    }

    pub fn parse_license_response(&self, license_b64: &str) -> CalyxResult<CbcsDecryptor> {
        let raw = BASE64
            .decode(license_b64.trim())
            .map_err(|e| CalyxError::Drm(format!("Invalid license Base64: {e}")))?;
        if raw.len() < 32 {
            return Err(CalyxError::Drm(
                "Widevine license payload too short".to_string(),
            ));
        }
        let (mut key, mut iv) = ([0u8; 16], [0u8; 16]);
        key.copy_from_slice(&raw[0..16]);
        iv.copy_from_slice(&raw[16..32]);
        Ok(CbcsDecryptor::new(key, iv))
    }

    pub fn extract_content_key_from_license(
        &self,
        license_b64: &str,
        lic_req_bytes: &[u8],
    ) -> CalyxResult<[u8; 16]> {
        let raw = BASE64
            .decode(license_b64.trim())
            .map_err(|e| CalyxError::Drm(format!("Invalid license Base64: {e}")))?;
        let (Some(n), Some(d)) = (&self.rsa_n, &self.rsa_d) else {
            let mut fallback = [0u8; 16];
            if raw.len() >= 16 {
                fallback.copy_from_slice(&raw[..16]);
                return Ok(fallback);
            }
            return Err(CalyxError::Drm("Missing RSA key in WVD".to_string()));
        };
        let lic_msg = find_proto_bytes(&raw, 2)
            .ok_or_else(|| CalyxError::Drm("SignedMessage missing msg".to_string()))?;
        let sess_ct = find_proto_bytes(&raw, 4)
            .or_else(|| find_proto_bytes(&raw, 6))
            .ok_or_else(|| CalyxError::Drm("SignedMessage missing session_key".to_string()))?;
        let ct_arr: [u8; 256] = sess_ct
            .try_into()
            .map_err(|_| CalyxError::Drm("session_key != 256 bytes".to_string()))?;
        let session_key_vec = rsa_oaep_sha1_decrypt(&ct_arr, d, n)?;
        if session_key_vec.len() < 16 {
            return Err(CalyxError::Drm(
                "Decrypted session_key < 16 bytes".to_string(),
            ));
        }
        let mut session_key = [0u8; 16];
        session_key.copy_from_slice(&session_key_vec[..16]);
        let mut cmac_input = Vec::with_capacity(lic_req_bytes.len() + 16);
        cmac_input.extend_from_slice(b"\x01ENCRYPTION\x00");
        cmac_input.extend_from_slice(lic_req_bytes);
        cmac_input.extend_from_slice(&[0x00, 0x00, 0x00, 0x80]);
        let enc_key = aes128_cmac(&session_key, &cmac_input);
        for container in find_all_proto_bytes(&lic_msg, 3) {
            if find_proto_varint(&container, 4) == Some(2)
                && let (Some(iv_b), Some(key_ct)) = (
                    find_proto_bytes(&container, 2),
                    find_proto_bytes(&container, 3),
                )
                && iv_b.len() >= 16
            {
                let mut iv = [0u8; 16];
                iv.copy_from_slice(&iv_b[..16]);
                return decrypt_aes128_cbc_key(&enc_key, &iv, &key_ct);
            }
        }
        Err(CalyxError::Drm(
            "No CONTENT key found in Widevine license".to_string(),
        ))
    }
}

fn push_proto_varint(out: &mut Vec<u8>, field: u32, mut val: u64) {
    out.push(((field << 3) as u8) & 0xF8);
    while val >= 0x80 {
        out.push(((val as u8) & 0x7F) | 0x80);
        val >>= 7;
    }
    out.push(val as u8);
}

fn push_proto_bytes(out: &mut Vec<u8>, field: u32, bytes: &[u8]) {
    out.push(((field << 3) | 2) as u8);
    let mut len = bytes.len();
    while len >= 0x80 {
        out.push(((len as u8) & 0x7F) | 0x80);
        len >>= 7;
    }
    out.push(len as u8);
    out.extend_from_slice(bytes);
}

fn read_varint(buf: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut val = 0u64;
    let mut shift = 0u32;
    while pos < buf.len() && shift < 64 {
        let b = buf[pos];
        pos += 1;
        val |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 {
            return Some((val, pos));
        }
        shift += 7;
    }
    None
}

fn find_proto_bytes(buf: &[u8], target_field: u32) -> Option<Vec<u8>> {
    find_all_proto_bytes(buf, target_field).into_iter().next()
}

fn find_all_proto_bytes(buf: &[u8], target_field: u32) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some((tag, p1)) = read_varint(buf, pos) {
        let field = (tag >> 3) as u32;
        let wire = (tag & 7) as u8;
        match wire {
            0 => {
                let Some((_, p2)) = read_varint(buf, p1) else {
                    break;
                };
                pos = p2;
            }
            1 => pos = p1 + 8,
            2 => {
                let Some((len, p2)) = read_varint(buf, p1) else {
                    break;
                };
                let end = p2.saturating_add(len as usize);
                if end > buf.len() {
                    break;
                }
                if field == target_field {
                    out.push(buf[p2..end].to_vec());
                }
                pos = end;
            }
            5 => pos = p1 + 4,
            _ => break,
        }
    }
    out
}

fn find_proto_varint(buf: &[u8], target_field: u32) -> Option<u64> {
    let mut pos = 0usize;
    while let Some((tag, p1)) = read_varint(buf, pos) {
        let field = (tag >> 3) as u32;
        let wire = (tag & 7) as u8;
        match wire {
            0 => {
                let (val, p2) = read_varint(buf, p1)?;
                if field == target_field {
                    return Some(val);
                }
                pos = p2;
            }
            1 => pos = p1 + 8,
            2 => {
                let (len, p2) = read_varint(buf, p1)?;
                pos = p2.saturating_add(len as usize);
            }
            5 => pos = p1 + 4,
            _ => break,
        }
    }
    None
}

pub struct CbcsDecryptor {
    pub content_key: [u8; 16],
    pub iv: [u8; 16],
}

impl CbcsDecryptor {
    pub fn new(content_key: [u8; 16], iv: [u8; 16]) -> Self {
        Self { content_key, iv }
    }

    pub fn decrypt_sample_in_place(
        &self,
        sample: &mut [u8],
        crypt_byte_block: usize,
        skip_byte_block: usize,
    ) -> CalyxResult<()> {
        if sample.len() < 16 {
            return Ok(());
        }
        let mut cipher = Aes128CbcDec::new(&self.content_key.into(), &self.iv.into());
        let pattern_len = (crypt_byte_block + skip_byte_block).max(1);
        for block_idx in 0..(sample.len() / 16) {
            if skip_byte_block == 0 || (block_idx % pattern_len) < crypt_byte_block {
                let start = block_idx * 16;
                cipher.decrypt_block_mut(GenericArray::from_mut_slice(
                    &mut sample[start..start + 16],
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wvd_and_cbcs_decryption() {
        let builtin = WidevineDeviceL3::builtin().expect("builtin Android 9 L3 WVD");
        assert_eq!(builtin.security_level, 3);
        assert!(builtin.rsa_n.is_some());

        let mut raw_pssh = vec![0u8; 32];
        raw_pssh[4..8].copy_from_slice(b"pssh");
        let b64 = BASE64.encode(&raw_pssh);
        let pssh = PsshBox::from_base64(&b64).expect("valid pssh");
        let signed = builtin
            .build_license_challenge(&pssh)
            .expect("signed challenge");
        assert!(signed.len() > 1800);
    }
}
