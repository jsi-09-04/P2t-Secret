//! storage/csv_import.rs - Importeur CSV standard pour P2TSecret.
//!
//! Format standard (correspondance par position) :
//!   - Colonne 1 (index 0) -> nom de l'application
//!   - Colonne 2 (index 1) -> login
//!   - Colonne 3 (index 2) -> mot de passe

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::crypto::SecretString;
use crate::storage::model::CredentialEntry;

/// Mot-clés indiquant qu'une ligne est un en-tête (colonne mot de passe).
const PASSWORD_HEADER_MARKERS: &[&str] =
    &["password", "pwd", "mot de passe", "motdepasse", "pass", "secret"];

/// Compteur garantissant des noms de fichiers temporaires uniques pour les tests.
#[cfg(test)]
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Analyse un fichier CSV selon le format standard :
/// nom, login, mot de passe disposés en colonnes 1, 2 et 3.
///
/// Le délimiteur (virgule `,` ou point-virgule `;`) est détecté automatiquement.
/// Une première ligne considérée comme un en-tête (mot de passe = mot-clé d'en-tête)
/// est ignorée. Les lignes incomplètes (nom ou mot de passe manquant) sont filtrées.
pub fn parse_csv_file(file_path: &Path) -> Result<Vec<CredentialEntry>, String> {
    if !file_path.exists() {
        return Err(format!("Le fichier n'existe pas : {}", file_path.display()));
    }

    let mut file = File::open(file_path).map_err(|e| format!("Erreur d'ouverture du fichier : {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Erreur de lecture du fichier : {}", e))?;

    // Détection du délimiteur le plus probable (virgule ou point-virgule).
    let first_line = content.lines().next().unwrap_or("");
    let delimiter = if first_line.split(';').count() > first_line.split(',').count() {
        b';'
    } else {
        b','
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let mut records: Vec<csv::StringRecord> = rdr.records().filter_map(|r| r.ok()).collect();

    // Ignorer une ligne d'en-tête si elle est présente.
    if let Some(first) = records.first() {
        if is_header_row(first) {
            records.remove(0);
        }
    }

    let mut entries = Vec::new();

    for record in records {
        // Correspondance positionnelle stricte : col 1 = nom, col 2 = login, col 3 = mot de passe.
        let name = record.get(0).unwrap_or("").trim();
        let login = record.get(1).unwrap_or("").trim();
        let password = record.get(2).unwrap_or("").trim();

        if name.is_empty() || password.is_empty() {
            continue;
        }

        entries.push(CredentialEntry::new(
            name,
            "",
            if login.is_empty() { "admin" } else { login },
            SecretString::new(password),
            "Général",
            Vec::new(),
            SecretString::new(""),
        ));
    }

    Ok(entries)
}

/// Indique si la ligne correspond à un en-tête (colonne mot de passe = mot-clé habituel).
fn is_header_row(record: &csv::StringRecord) -> bool {
    let pwd = record.get(2).unwrap_or("").trim().to_lowercase();
    PASSWORD_HEADER_MARKERS.contains(&pwd.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn parse_temp(csv: &str) -> Vec<CredentialEntry> {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("test_p2t_import_{}.csv", n));
        {
            let mut f = File::create(&temp_path).unwrap();
            f.write_all(csv.as_bytes()).unwrap();
        }
        let entries = parse_csv_file(&temp_path).unwrap();
        let _ = std::fs::remove_file(temp_path);
        entries
    }

    #[test]
    fn test_positional_comma() {
        let csv = "Google,admin@gmail.com,Secr3t!\nOVH,host,P@ssw0rd\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].title, "Google");
        assert_eq!(e[0].username, "admin@gmail.com");
        assert_eq!(e[0].password.as_str(), "Secr3t!");
        assert_eq!(e[1].title, "OVH");
        assert_eq!(e[1].password.as_str(), "P@ssw0rd");
    }

    #[test]
    fn test_positional_semicolon() {
        let csv = "Banque;user123;MonSuperMotDePasse\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "Banque");
        assert_eq!(e[0].username, "user123");
        assert_eq!(e[0].password.as_str(), "MonSuperMotDePasse");
    }

    #[test]
    fn test_skips_header_row() {
        let csv = "nom,login,mot de passe\nGoogle,g@t.com,Secr3t!\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "Google");
        assert_eq!(e[0].username, "g@t.com");
        assert_eq!(e[0].password.as_str(), "Secr3t!");
    }

    #[test]
    fn test_skips_incomplete_rows() {
        let csv = "SansMotDePasse,user,\nGoogle,g@t.com,Secr3t!\n,,\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "Google");
    }

    #[test]
    fn test_default_login_when_empty() {
        let csv = "App,,secret123\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].username, "admin");
        assert_eq!(e[0].password.as_str(), "secret123");
    }

    #[test]
    fn test_header_keyword_password_is_skipped() {
        let csv = "Service,login,password\nGitHub,john@example.com,hunter2\n";
        let e = parse_temp(csv);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "GitHub");
        assert_eq!(e[0].password.as_str(), "hunter2");
    }
}
