//! tests/crypto_tests.rs - Tests unitaires de la cryptographie et de la sécurité mémoire.

use p2tsecret::crypto::{
    decrypt_payload, derive_master_key, encrypt_payload, generate_password,
    generate_qr_matrix, generate_salt, PasswordOptions, TotpConfig,
};

#[test]
fn test_argon2id_derivation() {
    let password = "SuperMasterPassword2026!";
    let salt = generate_salt();

    let key1 = derive_master_key(password, &salt).expect("Dérivation 1");
    let key2 = derive_master_key(password, &salt).expect("Dérivation 2");

    assert_eq!(key1.0, key2.0);

    // Un sel différent produit une clé différente
    let salt2 = generate_salt();
    let key3 = derive_master_key(password, &salt2).expect("Dérivation 3");
    assert_ne!(key1.0, key3.0);
}

#[test]
fn test_aes_gcm_encryption_decryption() {
    let key = [42u8; 32];
    let secret = b"MonSecretUltraConfidentiel_P2T";

    let blob = encrypt_payload(secret, &key).expect("Chiffrement");
    assert_ne!(blob.ciphertext, secret);

    let decrypted = decrypt_payload(&blob, &key).expect("Déchiffrement");
    assert_eq!(decrypted, secret);
}

#[test]
fn test_aes_gcm_tampered_fails() {
    let key = [42u8; 32];
    let secret = b"SecretBancaire123";

    let mut blob = encrypt_payload(secret, &key).expect("Chiffrement");
    // Altération d'un octet du ciphertext
    blob.ciphertext[0] ^= 0xFF;

    let res = decrypt_payload(&blob, &key);
    assert!(res.is_err(), "Le déchiffrement doit échouer en cas d'altération");
}

#[test]
fn test_password_generator_entropy() {
    let opts = PasswordOptions {
        length: 24,
        use_upper: true,
        use_lower: true,
        use_digits: true,
        use_symbols: true,
        exclude_ambiguous: true,
    };

    let res = generate_password(&opts);
    assert_eq!(res.password.len(), 24);
    assert!(res.entropy_bits >= 80.0);
    assert_eq!(res.strength_label, "Très Robuste (Militaire)");
}

#[test]
fn test_totp_microsoft_authenticator() {
    let mut config = TotpConfig::new("james@projet2team.com").expect("Init TOTP");
    assert!(!config.secret_base32.is_empty());

    let url = config.get_url().expect("URL TOTP");
    assert!(url.starts_with("otpauth://totp/P2TSecret:james"));
    assert!(url.contains("issuer=P2TSecret"));

    // Vérification QR Code Matrix
    let (size, modules) = generate_qr_matrix(&url).expect("QR Matrix");
    assert!(size > 20);
    assert_eq!(modules.len(), size * size);

    // Validation du code courant généré par l'objet TOTP
    let totp = config.build_totp().expect("build_totp");
    let current_code = totp.generate_current().expect("generate_current");

    assert!(config.verify_code(&current_code));
    assert!(!config.verify_code("000000"));

    // Codes de secours
    let backup_codes = config.generate_backup_codes();
    assert_eq!(backup_codes.len(), 8);

    let first_code = &backup_codes[0];
    assert!(config.consume_backup_code(first_code));
    // Réutilisation du même code doit échouer
    assert!(!config.consume_backup_code(first_code));
}
