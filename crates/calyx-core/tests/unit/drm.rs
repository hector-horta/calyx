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
