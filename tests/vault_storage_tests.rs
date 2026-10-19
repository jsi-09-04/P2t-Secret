use p2tsecret::crypto::SecretString;
use p2tsecret::storage::{load_vault, save_vault, CredentialEntry, Vault};

#[test]
fn test_vault_save_and_reload() {
    let tmp_dir = std::env::temp_dir();
    let vault_file = tmp_dir.join(format!("test_vault_{}.p2t", rand::random::<u64>()));
    let master_password = "MyStrongVaultPassword2026!";

    // 1. Création d'un coffre en mémoire
    let mut vault = Vault::new("admin@p2t.test").expect("Nouveau Vault");

    let entry = CredentialEntry::new(
        "Serveur Dolibarr",
        "https://erp.projet2team.com",
        "dolibarr_admin",
        SecretString::new("DoliSecret987!#"),
        "Portails",
        vec!["erp".to_string(), "prod".to_string()],
        SecretString::new("Port SSH 2222"),
    );
    vault.entries.push(entry);

    // 2. Sauvegarde atomique chiffrée
    save_vault(&vault, master_password, &vault_file).expect("Sauvegarde");
    assert!(vault_file.exists());

    // 3. Rechargement avec le bon mot de passe
    let reloaded = load_vault(&vault_file, master_password).expect("Chargement réussi");
    assert_eq!(reloaded.entries.len(), 1);
    assert_eq!(reloaded.entries[0].title, "Serveur Dolibarr");
    assert_eq!(reloaded.entries[0].password.as_str(), "DoliSecret987!#");
    assert_eq!(reloaded.entries[0].notes.as_str(), "Port SSH 2222");

    // 4. Tentative de chargement avec mauvais mot de passe
    let bad_load = load_vault(&vault_file, "MauvaisMotDePasse!");
    assert!(bad_load.is_err(), "Le déchiffrement avec un mauvais mot de passe doit échouer");

    // Nettoyage
    let _ = std::fs::remove_file(&vault_file);
}
