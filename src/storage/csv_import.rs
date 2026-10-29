//! storage/csv_import.rs - Module d'importation CSV universel et tolérant pour P2TSecret.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::crypto::SecretString;
use crate::storage::model::CredentialEntry;

/// Résultat de l'analyse préalable d'un fichier CSV
#[derive(Clone, Debug)]
pub struct CsvImportPreview {
    pub total_found: usize,
    pub sample_entries: Vec<CredentialEntry>,
}

/// Résultat effectif de l'importation
#[derive(Clone, Debug, Default)]
pub struct CsvImportResult {
    pub imported_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
}

/// Analyse et extrait les identifiants d'un fichier CSV.
/// Gère automatiquement la détection des délimiteurs (virgule `,` ou point-virgule `;`)
/// et la correspondance souple des noms de colonnes.
pub fn parse_csv_file(file_path: &Path) -> Result<Vec<CredentialEntry>, String> {
    if !file_path.exists() {
        return Err(format!("Le fichier n'existe pas : {}", file_path.display()));
    }

    let mut file = File::open(file_path).map_err(|e| format!("Erreur d'ouverture du fichier : {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Erreur de lecture du fichier : {}", e))?;

    // Détection du délimiteur le plus probable
    let comma_count = content.lines().next().unwrap_or("").matches(',').count();
    let semi_count = content.lines().next().unwrap_or("").matches(';').count();
    let delimiter = if semi_count > comma_count { b';' } else { b',' };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers = rdr.headers().map_err(|e| format!("Erreur de lecture des en-têtes CSV : {}", e))?.clone();

    // Recherche des indices de colonnes (insensible à la casse et tolérant)
    let mut col_title: Option<usize> = None;
    let mut col_url: Option<usize> = None;
    let mut col_user: Option<usize> = None;
    let mut col_pwd: Option<usize> = None;
    let mut col_cat: Option<usize> = None;
    let mut col_tags: Option<usize> = None;
    let mut col_notes: Option<usize> = None;

    for (idx, h) in headers.iter().enumerate() {
        let norm = h.to_lowercase();
        if col_title.is_none() && (norm.contains("title") || norm.contains("service") || norm.contains("nom") || norm.contains("name")) {
            col_title = Some(idx);
        } else if col_url.is_none() && (norm.contains("url") || norm.contains("site") || norm.contains("lien") || norm.contains("link")) {
            col_url = Some(idx);
        } else if col_user.is_none() && (norm.contains("user") || norm.contains("login") || norm.contains("email") || norm.contains("identifiant")) {
            col_user = Some(idx);
        } else if col_pwd.is_none() && (norm.contains("pass") || norm.contains("pwd") || norm.contains("mot de passe") || norm.contains("secret")) {
            col_pwd = Some(idx);
        } else if col_cat.is_none() && (norm.contains("cat") || norm.contains("catégorie") || norm.contains("categorie")) {
            col_cat = Some(idx);
        } else if col_tags.is_none() && (norm.contains("tag") || norm.contains("étiquette") || norm.contains("label")) {
            col_tags = Some(idx);
        } else if col_notes.is_none() && (norm.contains("note") || norm.contains("comment") || norm.contains("remarque")) {
            col_notes = Some(idx);
        }
    }

    // Si le titre ou le mot de passe est introuvable par en-tête, tentative de fallback positionnel
    let idx_title = col_title.unwrap_or(0);
    let idx_pwd = col_pwd.ok_or_else(|| "Colonne mot de passe ('password', 'pwd', 'mot de passe') introuvable dans le CSV.".to_string())?;
    let idx_user = col_user.unwrap_or(1);

    let mut entries = Vec::new();

    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        let title = record.get(idx_title).unwrap_or("").trim();
        let password = record.get(idx_pwd).unwrap_or("").trim();
        let username = record.get(idx_user).unwrap_or("").trim();

        if title.is_empty() || password.is_empty() {
            continue;
        }

        let url = col_url.and_then(|i| record.get(i)).unwrap_or("").trim();
        let category = col_cat
            .and_then(|i| record.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("Général");

        let tags_str = col_tags.and_then(|i| record.get(i)).unwrap_or("").trim();
        let tags: Vec<String> = if !tags_str.is_empty() {
            tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        } else {
            Vec::new()
        };

        let notes = col_notes.and_then(|i| record.get(i)).unwrap_or("").trim();

        let entry = CredentialEntry::new(
            title,
            url,
            if username.is_empty() { "admin" } else { username },
            SecretString::new(password),
            category,
            tags,
            SecretString::new(notes),
        );

        entries.push(entry);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_csv_import_comma() {
        let csv_data = "title,url,username,password,category,tags,notes\n\
Google,https://google.com,test@gmail.com,Secr3t!,Cloud,\"Search, Gmail\",Compte pro\n\
OVH,https://ovh.com,admin,P@ssw0rd,Serveurs,Hébergement,VPS";

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_p2t_import.csv");
        {
            let mut f = File::create(&temp_path).unwrap();
            f.write_all(csv_data.as_bytes()).unwrap();
        }

        let entries = parse_csv_file(&temp_path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Google");
        assert_eq!(entries[0].username, "test@gmail.com");
        assert_eq!(entries[0].password.as_str(), "Secr3t!");
        assert_eq!(entries[0].category, "Cloud");
        assert_eq!(entries[1].title, "OVH");

        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_csv_import_semicolon() {
        let csv_data = "Service;Login;Mot de passe;Catégorie\n\
Banque;user123;MonSuperMotDePasse;Finance";

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_p2t_import_semi.csv");
        {
            let mut f = File::create(&temp_path).unwrap();
            f.write_all(csv_data.as_bytes()).unwrap();
        }

        let entries = parse_csv_file(&temp_path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Banque");
        assert_eq!(entries[0].username, "user123");
        assert_eq!(entries[0].password.as_str(), "MonSuperMotDePasse");
        assert_eq!(entries[0].category, "Finance");

        let _ = std::fs::remove_file(temp_path);
    }
}
