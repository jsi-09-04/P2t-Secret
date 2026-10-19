//! crypto/generator.rs - Générateur cryptographique de mots de passe et calcul d'entropie.

use rand::seq::SliceRandom;

#[derive(Clone, Debug)]
pub struct PasswordOptions {
    pub length: usize,
    pub use_upper: bool,
    pub use_lower: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
    pub exclude_ambiguous: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            use_upper: true,
            use_lower: true,
            use_digits: true,
            use_symbols: true,
            exclude_ambiguous: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PasswordResult {
    pub password: String,
    pub entropy_bits: f64,
    pub strength_label: &'static str,
}

pub fn generate_password(opts: &PasswordOptions) -> PasswordResult {
    let mut upper: Vec<char> = ('A'..='Z').collect();
    let mut lower: Vec<char> = ('a'..='z').collect();
    let mut digits: Vec<char> = ('0'..='9').collect();
    let mut symbols: Vec<char> = "!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect();

    if opts.exclude_ambiguous {
        let ambiguous = ['0', 'O', 'o', 'l', '1', 'I', '|', '`', '\'', '"', ';', ':'];
        upper.retain(|c| !ambiguous.contains(c));
        lower.retain(|c| !ambiguous.contains(c));
        digits.retain(|c| !ambiguous.contains(c));
        symbols.retain(|c| !ambiguous.contains(c));
    }

    let mut pool: Vec<char> = Vec::new();
    let mut mandatory: Vec<char> = Vec::new();
    let mut rng = rand::thread_rng();

    if opts.use_upper && !upper.is_empty() {
        mandatory.push(*upper.choose(&mut rng).unwrap());
        pool.extend(upper);
    }
    if opts.use_lower && !lower.is_empty() {
        mandatory.push(*lower.choose(&mut rng).unwrap());
        pool.extend(lower);
    }
    if opts.use_digits && !digits.is_empty() {
        mandatory.push(*digits.choose(&mut rng).unwrap());
        pool.extend(digits);
    }
    if opts.use_symbols && !symbols.is_empty() {
        mandatory.push(*symbols.choose(&mut rng).unwrap());
        pool.extend(symbols);
    }

    if pool.is_empty() {
        pool.extend(('a'..='z').chain('0'..='9'));
    }

    let target_len = opts.length.clamp(8, 128);
    let remaining = target_len.saturating_sub(mandatory.len());

    let mut chars = mandatory;
    for _ in 0..remaining {
        chars.push(*pool.choose(&mut rng).unwrap());
    }

    chars.shuffle(&mut rng);
    let password: String = chars.into_iter().collect();

    // Calcul de l'entropie : L * log2(pool_size)
    let pool_size = pool.len().max(2) as f64;
    let entropy_bits = (target_len as f64) * pool_size.log2();

    let strength_label = if entropy_bits >= 80.0 {
        "Très Robuste (Militaire)"
    } else if entropy_bits >= 60.0 {
        "Robuste"
    } else if entropy_bits >= 45.0 {
        "Moyen"
    } else {
        "Faible"
    };

    PasswordResult {
        password,
        entropy_bits: (entropy_bits * 10.0).round() / 10.0,
        strength_label,
    }
}
