/**
 * P2TSecret - Logique Frontend Moderne & Gestionnaire 2FA Microsoft Authenticator
 */

const API_BASE = '/api/v1';

// État Global de l'application
const state = {
  token: localStorage.getItem('p2t_token') || null,
  user: null,
  tempToken: null,
  credentials: [],
  activeTab: 'vault',
};

// ==========================================
// Notifications Toast
// ==========================================
function showToast(message, type = 'success') {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.innerHTML = `<span>${type === 'success' ? '🛡️' : '⚠️'}</span><span>${message}</span>`;
  container.appendChild(toast);

  setTimeout(() => {
    toast.style.opacity = '0';
    setTimeout(() => toast.remove(), 300);
  }, 3500);
}

// ==========================================
// Copie sécurisée dans le presse-papier avec purge
// ==========================================
async function copyToClipboardSecure(text, label = "Mot de passe") {
  try {
    await navigator.clipboard.writeText(text);
    showToast(`${label} copié ! Effacement auto du presse-papier dans 30s.`, 'success');

    // Purge de sécurité après 30s
    setTimeout(async () => {
      try {
        const current = await navigator.clipboard.readText();
        if (current === text) {
          await navigator.clipboard.writeText('');
          showToast('Presse-papier sécurisé et effacé.', 'success');
        }
      } catch (_) {}
    }, 30000);
  } catch (err) {
    showToast('Erreur lors de la copie dans le presse-papier.', 'error');
  }
}

// ==========================================
// Appels API Authentifiés
// ==========================================
async function apiCall(endpoint, method = 'GET', body = null, token = state.token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${API_BASE}${endpoint}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : null,
  });

  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.detail || 'Une erreur est survenue.');
  }
  return data;
}

// ==========================================
// Initialisation & Détection d'Auth
// ==========================================
document.addEventListener('DOMContentLoaded', async () => {
  setupEventListeners();
  if (state.token) {
    try {
      state.user = await apiCall('/auth/me');
      showVaultView();
    } catch (e) {
      logout();
    }
  } else {
    showAuthView();
  }
});

function showAuthView() {
  document.getElementById('auth-section').style.display = 'block';
  document.getElementById('vault-section').style.display = 'none';
  document.getElementById('nav-user-info').style.display = 'none';
}

function showVaultView() {
  document.getElementById('auth-section').style.display = 'none';
  document.getElementById('vault-section').style.display = 'block';
  document.getElementById('nav-user-info').style.display = 'flex';
  document.getElementById('nav-username').textContent = state.user?.username || 'Utilisateur';

  const badge2FA = document.getElementById('badge-2fa-status');
  if (state.user?.is_2fa_enabled) {
    badge2FA.innerHTML = '🛡️ 2FA Microsoft Authenticator Actif';
    badge2FA.className = 'badge badge-2fa';
  } else {
    badge2FA.innerHTML = '⚠️ 2FA Recommandé';
    badge2FA.className = 'badge badge-enc';
  }

  loadCredentials();
}

function logout() {
  state.token = null;
  state.user = null;
  localStorage.removeItem('p2t_token');
  showAuthView();
  showToast('Déconnexion effectuée avec succès.');
}

// ==========================================
// Événements & Gestion des Formulaires
// ==========================================
function setupEventListeners() {
  // Onglets Login / Register
  const tabLogin = document.getElementById('tab-login');
  const tabRegister = document.getElementById('tab-register');
  const formLogin = document.getElementById('form-login');
  const formRegister = document.getElementById('form-register');

  tabLogin?.addEventListener('click', () => {
    tabLogin.classList.add('active');
    tabRegister.classList.remove('active');
    formLogin.style.display = 'block';
    formRegister.style.display = 'none';
  });

  tabRegister?.addEventListener('click', () => {
    tabRegister.classList.add('active');
    tabLogin.classList.remove('active');
    formLogin.style.display = 'none';
    formRegister.style.display = 'block';
  });

  // Soumission Formulaire Inscription
  formRegister?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const username = document.getElementById('reg-username').value.trim();
    const password = document.getElementById('reg-password').value;
    const confirm = document.getElementById('reg-password-confirm').value;

    if (password !== confirm) {
      showToast('Les mots de passe ne correspondent pas.', 'error');
      return;
    }

    try {
      await apiCall('/auth/register', 'POST', { username, password });
      showToast('Compte créé ! Connexion en cours...', 'success');
      // Login direct pour initier la 2FA
      const loginRes = await apiCall('/auth/login', 'POST', { username, password });
      state.token = loginRes.access_token;
      state.user = { id: loginRes.user_id, username: loginRes.username };
      open2FASetupModal();
    } catch (err) {
      showToast(err.message, 'error');
    }
  });

  // Soumission Formulaire Connexion
  formLogin?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const username = document.getElementById('login-username').value.trim();
    const password = document.getElementById('login-password').value;

    try {
      const res = await apiCall('/auth/login', 'POST', { username, password });
      if (res.requires_2fa) {
        state.tempToken = res.temp_token;
        open2FAChallengeModal(username);
      } else {
        // Utilisateur connecté sans 2FA encore activée
        state.token = res.access_token;
        localStorage.setItem('p2t_token', state.token);
        state.user = await apiCall('/auth/me');
        showVaultView();
        showToast('Connexion réussie. Pensez à activer le 2FA !');
      }
    } catch (err) {
      showToast(err.message, 'error');
    }
  });

  // Vérification TOTP Challenge 2FA
  document.getElementById('form-2fa-verify')?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const code = document.getElementById('totp-challenge-code').value.trim();
    try {
      const res = await apiCall('/auth/2fa/verify', 'POST', {
        temp_token: state.tempToken,
        code: code,
      });
      state.token = res.access_token;
      localStorage.setItem('p2t_token', state.token);
      state.user = await apiCall('/auth/me');
      closeModal('modal-2fa-challenge');
      showVaultView();
      showToast('Authentification 2FA Microsoft Authenticator validée !', 'success');
    } catch (err) {
      showToast(err.message, 'error');
    }
  });

  // Bouton Déconnexion
  document.getElementById('btn-logout')?.addEventListener('click', logout);

  // Recherche en temps réel dans le coffre
  document.getElementById('vault-search')?.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    filterCredentials(query);
  });

  // Modale Nouvel Identifiant
  document.getElementById('btn-new-cred')?.addEventListener('click', () => {
    openCredentialModal();
  });

  // Générateur de mot de passe autonome
  document.getElementById('btn-open-generator')?.addEventListener('click', () => {
    openModal('modal-generator');
    triggerPasswordGenerator();
  });

  // Audit Logs
  document.getElementById('btn-open-audit')?.addEventListener('click', openAuditModal);

  // Soumission Formulaire Identifiant (Ajout / Modification)
  document.getElementById('form-cred')?.addEventListener('submit', handleSaveCredential);

  // Boutons du Générateur de mot de passe
  document.getElementById('gen-length')?.addEventListener('input', (e) => {
    document.getElementById('gen-length-val').textContent = e.target.value;
    triggerPasswordGenerator();
  });
  ['gen-upper', 'gen-lower', 'gen-digits', 'gen-symbols', 'gen-no-ambiguous'].forEach(id => {
    document.getElementById(id)?.addEventListener('change', triggerPasswordGenerator);
  });
  document.getElementById('btn-refresh-pwd')?.addEventListener('click', triggerPasswordGenerator);
  document.getElementById('btn-copy-gen-pwd')?.addEventListener('click', () => {
    const pwd = document.getElementById('gen-output').value;
    copyToClipboardSecure(pwd, "Mot de passe généré");
  });
}

// ==========================================
// Enrôlement 2FA Microsoft Authenticator
// ==========================================
async function open2FASetupModal() {
  try {
    const res = await apiCall('/auth/2fa/setup', 'POST');
    document.getElementById('setup-qr-code').innerHTML = res.qr_svg;
    document.getElementById('setup-secret-key').textContent = res.secret;
    document.getElementById('setup-otpauth-link').href = res.otpauth_url;
    openModal('modal-2fa-setup');
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Confirmation 2FA Setup
async function confirm2FASetup() {
  const code = document.getElementById('setup-totp-code').value.trim();
  if (code.length !== 6) {
    showToast('Veuillez saisir le code à 6 chiffres de Microsoft Authenticator.', 'error');
    return;
  }

  try {
    const res = await apiCall('/auth/2fa/confirm', 'POST', { code });
    state.token = res.access_token;
    localStorage.setItem('p2t_token', state.token);
    state.user = await apiCall('/auth/me');

    // Afficher les codes de secours
    document.getElementById('setup-step-1').style.display = 'none';
    document.getElementById('setup-step-2').style.display = 'block';

    const grid = document.getElementById('backup-codes-container');
    grid.innerHTML = res.backup_codes.map(c => `<div class="backup-code-item">${c}</div>`).join('');

    showToast('2FA Microsoft Authenticator configuré !', 'success');
  } catch (err) {
    showToast(err.message, 'error');
  }
}

function finish2FASetup() {
  closeModal('modal-2fa-setup');
  showVaultView();
}

function open2FAChallengeModal(username) {
  document.getElementById('challenge-username-display').textContent = username;
  document.getElementById('totp-challenge-code').value = '';
  openModal('modal-2fa-challenge');
  setTimeout(() => document.getElementById('totp-challenge-code').focus(), 100);
}

// ==========================================
// Gestion du Coffre-fort (Credentials)
// ==========================================
async function loadCredentials() {
  try {
    state.credentials = await apiCall('/vault');
    renderCredentials(state.credentials);
    updateStats(state.credentials);
  } catch (err) {
    showToast(err.message, 'error');
  }
}

function updateStats(items) {
  document.getElementById('stat-total-secrets').textContent = items.length;
  const categories = new Set(items.map(i => i.category || 'Général'));
  document.getElementById('stat-total-categories').textContent = categories.size;
}

function renderCredentials(items) {
  const grid = document.getElementById('credentials-grid');
  if (items.length === 0) {
    grid.innerHTML = `
      <div style="grid-column: 1 / -1; text-align: center; padding: 4rem 1rem; color: var(--text-dim);">
        <div style="font-size: 3rem; margin-bottom: 1rem;">🔒</div>
        <h3>Aucun identifiant pour le moment</h3>
        <p style="margin-top: 0.5rem;">Cliquez sur <strong>+ Nouvel Identifiant</strong> pour stocker votre premier secret en toute sécurité.</p>
      </div>
    `;
    return;
  }

  grid.innerHTML = items.map(c => `
    <div class="credential-card glass" data-id="${c.id}">
      <div class="cred-header">
        <div>
          <h3 class="cred-title">${escapeHtml(c.title)}</h3>
          ${c.url ? `<a href="${escapeHtml(c.url)}" target="_blank" rel="noopener noreferrer" class="cred-url">${escapeHtml(c.url.replace(/^https?:\/\//, ''))} ↗</a>` : ''}
        </div>
        <span class="cred-category-badge">${escapeHtml(c.category || 'Général')}</span>
      </div>

      <div class="cred-body">
        <div class="cred-field">
          <span style="color: var(--text-dim); font-size: 0.8rem;">Login :</span>
          <span class="cred-field-value">${escapeHtml(c.username)}</span>
          <div class="cred-field-actions">
            <button class="icon-btn" onclick="copyToClipboardSecure('${escapeJs(c.username)}', 'Identifiant')" title="Copier le login">📋</button>
          </div>
        </div>

        <div class="cred-field">
          <span style="color: var(--text-dim); font-size: 0.8rem;">Mot de passe :</span>
          <span class="cred-field-value" id="pwd-display-${c.id}">••••••••••••</span>
          <div class="cred-field-actions">
            <button class="icon-btn" onclick="toggleRevealPassword('${c.id}')" title="Démasquer">👁️</button>
            <button class="icon-btn" onclick="copyDecryptedPassword('${c.id}')" title="Copier le mot de passe">📋</button>
          </div>
        </div>
      </div>

      <div class="cred-footer">
        <span>${c.tags ? escapeHtml(c.tags) : 'Chiffré AES-256-GCM'}</span>
        <div style="display: flex; gap: 0.5rem;">
          <button class="icon-btn" onclick="editCredential('${c.id}')" title="Modifier">✏️</button>
          <button class="icon-btn" onclick="deleteCredential('${c.id}')" style="color: var(--accent-rose);" title="Supprimer">🗑️</button>
        </div>
      </div>
    </div>
  `).join('');
}

function filterCredentials(query) {
  const filtered = state.credentials.filter(c => 
    c.title.toLowerCase().includes(query) ||
    c.username.toLowerCase().includes(query) ||
    (c.url && c.url.toLowerCase().includes(query)) ||
    (c.category && c.category.toLowerCase().includes(query)) ||
    (c.tags && c.tags.toLowerCase().includes(query))
  );
  renderCredentials(filtered);
}

// Démasquage temporaire
async function toggleRevealPassword(credId) {
  const elem = document.getElementById(`pwd-display-${credId}`);
  if (elem.dataset.revealed === "true") {
    elem.textContent = '••••••••••••';
    elem.dataset.revealed = "false";
  } else {
    try {
      const detail = await apiCall(`/vault/${credId}`);
      elem.textContent = detail.password;
      elem.dataset.revealed = "true";
      // Masquer automatiquement après 15 secondes
      setTimeout(() => {
        if (elem.dataset.revealed === "true") {
          elem.textContent = '••••••••••••';
          elem.dataset.revealed = "false";
        }
      }, 15000);
    } catch (err) {
      showToast(err.message, 'error');
    }
  }
}

// Copie du mot de passe déchiffré
async function copyDecryptedPassword(credId) {
  try {
    const detail = await apiCall(`/vault/${credId}`);
    copyToClipboardSecure(detail.password, "Mot de passe");
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Modal Création / Édition
function openCredentialModal(cred = null) {
  document.getElementById('form-cred-id').value = cred?.id || '';
  document.getElementById('cred-modal-title').textContent = cred ? "Modifier l'identifiant" : "Ajouter un identifiant sécurisé";
  document.getElementById('cred-title').value = cred?.title || '';
  document.getElementById('cred-url').value = cred?.url || '';
  document.getElementById('cred-username').value = cred?.username || '';
  document.getElementById('cred-password').value = '';
  document.getElementById('cred-password').placeholder = cred ? "Laisser vide pour ne pas modifier" : "Mot de passe secret";
  document.getElementById('cred-category').value = cred?.category || 'Général';
  document.getElementById('cred-tags').value = cred?.tags || '';
  document.getElementById('cred-notes').value = cred?.notes || '';
  openModal('modal-credential');
}

async function editCredential(credId) {
  try {
    const detail = await apiCall(`/vault/${credId}`);
    openCredentialModal(detail);
  } catch (err) {
    showToast(err.message, 'error');
  }
}

async function deleteCredential(credId) {
  if (!confirm("Êtes-vous sûr de vouloir supprimer cet identifiant définitivement ?")) return;
  try {
    await apiCall(`/vault/${credId}`, 'DELETE');
    showToast('Identifiant supprimé.');
    loadCredentials();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

async function handleSaveCredential(e) {
  e.preventDefault();
  const id = document.getElementById('form-cred-id').value;
  const payload = {
    title: document.getElementById('cred-title').value.trim(),
    url: document.getElementById('cred-url').value.trim(),
    username: document.getElementById('cred-username').value.trim(),
    category: document.getElementById('cred-category').value.trim(),
    tags: document.getElementById('cred-tags').value.trim(),
    notes: document.getElementById('cred-notes').value.trim(),
  };

  const passwordVal = document.getElementById('cred-password').value;
  if (passwordVal) payload.password = passwordVal;

  try {
    if (id) {
      await apiCall(`/vault/${id}`, 'PUT', payload);
      showToast('Identifiant mis à jour avec succès.');
    } else {
      if (!payload.password) {
        showToast('Le mot de passe est obligatoire pour la création.', 'error');
        return;
      }
      await apiCall('/vault', 'POST', payload);
      showToast('Nouvel identifiant chiffré et sauvegardé.');
    }
    closeModal('modal-credential');
    loadCredentials();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// ==========================================
// Générateur de Mot de Passe Cryptographique
// ==========================================
async function triggerPasswordGenerator() {
  const payload = {
    length: parseInt(document.getElementById('gen-length').value, 10),
    use_upper: document.getElementById('gen-upper').checked,
    use_lower: document.getElementById('gen-lower').checked,
    use_digits: document.getElementById('gen-digits').checked,
    use_symbols: document.getElementById('gen-symbols').checked,
    exclude_ambiguous: document.getElementById('gen-no-ambiguous').checked,
  };

  try {
    const res = await apiCall('/vault/generate-password', 'POST', payload);
    document.getElementById('gen-output').value = res.password;
    document.getElementById('gen-entropy-display').textContent = `${res.info.entropy_bits} bits — ${res.info.strength}`;
  } catch (err) {
    showToast(err.message, 'error');
  }
}

function useGeneratedPasswordInForm() {
  const pwd = document.getElementById('gen-output').value;
  const input = document.getElementById('cred-password');
  if (input) input.value = pwd;
  closeModal('modal-generator');
  showToast('Mot de passe injecté dans le formulaire.');
}

// ==========================================
// Journal d'Audit
// ==========================================
async function openAuditModal() {
  try {
    const logs = await apiCall('/vault/audit');
    const container = document.getElementById('audit-logs-list');
    if (logs.length === 0) {
      container.innerHTML = '<p style="color: var(--text-dim); text-align: center;">Aucun événement enregistré.</p>';
    } else {
      container.innerHTML = logs.map(l => `
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.65rem 0; border-bottom: 1px solid var(--border-glass); font-size: 0.85rem;">
          <div>
            <span style="font-weight: 600; color: ${l.status === 'success' ? 'var(--accent-emerald)' : 'var(--accent-rose)'};">${l.action}</span>
            ${l.target_title ? `<span style="color: var(--text-muted); margin-left: 0.5rem;">(${escapeHtml(l.target_title)})</span>` : ''}
          </div>
          <span style="color: var(--text-dim); font-size: 0.75rem;">${new Date(l.timestamp).toLocaleString()}</span>
        </div>
      `).join('');
    }
    openModal('modal-audit');
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// ==========================================
// Utilitaires Modales & Sécurité
// ==========================================
function openModal(id) {
  document.getElementById(id)?.classList.add('open');
}

function closeModal(id) {
  document.getElementById(id)?.classList.remove('open');
}

function escapeHtml(str) {
  if (!str) return '';
  return str.replace(/[&<>"']/g, m => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;'
  })[m]);
}

function escapeJs(str) {
  if (!str) return '';
  return str.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/"/g, '\\"');
}
