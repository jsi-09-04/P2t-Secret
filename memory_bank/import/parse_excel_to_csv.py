#!/usr/bin/env python3
"""
Convertisseur d'inventaire Excel vers CSV pour P2TSecret.
Lit toutes les feuilles du fichier 'Liste des comptes de messagerie_v3_2023.xlsx'
et génère un fichier 'comptes_p2tsecret.csv' structuré et nettoyé.
"""

import csv
import os
import openpyxl

EXCEL_PATH = "/Users/james/02-Interne/P2T Applications/P2TSecret/memory_bank/import/Liste des comptes de messagerie_v3_2023.xlsx"
CSV_PATH = "/Users/james/02-Interne/P2T Applications/P2TSecret/memory_bank/import/comptes_p2tsecret.csv"

def determine_category(title, username, notes):
    t = f"{title} {username} {notes}".lower()
    if any(k in t for k in ["banque", "crédit mutuel", "bourse direct", "fortunéo", "fortuneo", "ig", "fxcm", "stripe", "épargne", "epargne", "mollie"]):
        return "Finance"
    elif any(k in t for k in ["ovh", "ionos", "office 365", "qnap", "google", "prestashop", "shopify", "hosting", "webd", "cpanel", "apple", "cloud", "aws", "azure"]):
        return "Cloud"
    elif any(k in t for k in ["nas", "serveur", "server", "mysql", "postgresql", "ssh", "wifi", "routeur", "router", "rsync", "réseau", "reseau"]):
        return "Serveurs"
    elif any(k in t for k in ["messagerie", "mail", "gmail", "outlook", "thunderbird", "support@", "noreply@", "contact@"]):
        return "Messagerie"
    elif any(k in t for k in ["cgi", "dolibarr", "urssaf", "insee", "sage", "datadoc", "jmg", "alca", "lemenu"]):
        return "Pro"
    elif any(k in t for k in ["doctolib", "amazon", "fnac", "sncf", "zalando", "freebox", "free", "ryanair", "booking", "le bon coin", "tondeuse", "ameli"]):
        return "Personnel"
    return "Général"

def clean_str(val):
    if val is None:
        return ""
    s = str(val).strip()
    return s

def run():
    wb = openpyxl.load_workbook(EXCEL_PATH, data_only=True)
    records = []
    seen = set()

    def add_entry(title, url, username, password, category=None, tags="", notes=""):
        t_clean = clean_str(title)
        u_clean = clean_str(username)
        p_clean = clean_str(password)
        if not t_clean or not p_clean:
            return
        if not u_clean:
            u_clean = "admin"
        if p_clean.lower() in ["pwd", "mot de passe", "password"]:
            return

        key = (t_clean.lower(), u_clean.lower())
        if key in seen:
            return
        seen.add(key)

        cat = category or determine_category(t_clean, u_clean, notes)
        records.append({
            "title": t_clean,
            "url": clean_str(url),
            "username": u_clean,
            "password": p_clean,
            "category": cat,
            "tags": clean_str(tags),
            "notes": clean_str(notes)
        })

    # ==========================================
    # 1. Comptes de messagerie
    # ==========================================
    if "Comptes de messagerie" in wb.sheetnames:
        s = wb["Comptes de messagerie"]
        for r in range(3, s.max_row + 1):
            nas = s.cell(r, 1).value
            reseau_u = s.cell(r, 3).value
            reseau_p = s.cell(r, 4).value
            ms_u = s.cell(r, 6).value
            ms_p = s.cell(r, 7).value
            nom = s.cell(r, 8).value
            mail_u = s.cell(r, 10).value
            mail_p = s.cell(r, 11).value
            fb = s.cell(r, 12).value
            doli_u = s.cell(r, 22).value
            doli_p = s.cell(r, 23).value

            if mail_u and mail_p:
                add_entry(f"Messagerie P2T - {nom or mail_u}", "https://mail.ovh.net", mail_u, mail_p, "Messagerie", "P2T, OVH, Email", f"Réseau: {reseau_u or ''}")
            if ms_u and ms_p:
                add_entry(f"Office 365 P2T - {nom or ms_u}", "https://portal.office.com", ms_u, ms_p, "Cloud", "P2T, Office 365", "")
            if reseau_u and reseau_p:
                cat = "Serveurs" if nas or "nas" in str(reseau_u).lower() else "Général"
                nom_srv = nas.strip() if (nas and nas.strip()) else str(reseau_u).strip()
                add_entry(f"Réseau P2T - {nom_srv}", "", reseau_u, reseau_p, cat, "Réseau, Local", f"Equipement: {nas or ''}")
            if doli_u and doli_p:
                add_entry(f"Dolibarr P2T - {nom or doli_u}", "https://p2t.projet2team.fr", doli_u, doli_p, "Pro", "Dolibarr, ERP", "")

    # ==========================================
    # 2. Autres Pwd
    # ==========================================
    if "Autres Pwd" in wb.sheetnames:
        s = wb["Autres Pwd"]
        cur_srv = ""
        for r in range(1, s.max_row + 1):
            c1 = clean_str(s.cell(r, 1).value)
            c2 = clean_str(s.cell(r, 2).value)
            c3 = clean_str(s.cell(r, 3).value)
            c4 = clean_str(s.cell(r, 4).value)
            c5 = clean_str(s.cell(r, 5).value)
            c6 = clean_str(s.cell(r, 6).value)
            c7 = clean_str(s.cell(r, 7).value)
            c8 = clean_str(s.cell(r, 8).value)

            if "login" in c3.lower() or "pwd" in c5.lower() or "liste des" in c2.lower() or "*****" in c2:
                continue

            title = c2 or c1 or cur_srv
            if c2:
                cur_srv = c2

            # Identification login & pwd
            pwd = ""
            user = ""
            notes = []

            # Choix du mot de passe le plus récent ou primaire
            for v in [c6, c5, c4, c7]:
                if v and v.lower() not in ["pwd", "ancien", "nouveau", "actionnariat"]:
                    if not pwd:
                        pwd = v
                    elif v != pwd and not user and "@" in v:
                        user = v
                    elif v != pwd:
                        notes.append(f"Info/Alt: {v}")

            for v in [c3, c4, c2]:
                if v and v != pwd and v.lower() not in ["login", "pwd", "compte"]:
                    if not user and v != title:
                        user = v

            if pwd:
                add_entry(title, "", user or "james.simon", pwd, None, "", " | ".join(notes))

    # ==========================================
    # 3. Codes secours OVH
    # ==========================================
    if "Codes secours OVH" in wb.sheetnames:
        s = wb["Codes secours OVH"]
        for r in range(1, s.max_row + 1):
            val = clean_str(s.cell(r, 1).value)
            if val and "-" in val and len(val) >= 16:
                add_entry(f"Code Secours OVH #{r}", "https://www.ovh.com", "sj102551-ovh", val, "Cloud", "OVH, 2FA, Backup", "Code d'urgence OVH")

    # ==========================================
    # 4. Ledoux
    # ==========================================
    if "Ledoux" in wb.sheetnames:
        s = wb["Ledoux"]
        for r in range(1, s.max_row + 1):
            mail = clean_str(s.cell(r, 2).value)
            pwd = clean_str(s.cell(r, 3).value)
            if mail and pwd and "@" in mail:
                add_entry(f"Fondation Ledoux - {mail}", "https://mail.ovh.net", mail, pwd, "Messagerie", "Ledoux, Email", "")

    # ==========================================
    # 5. Sheet 048 (JMG Engineering)
    # ==========================================
    if "048" in wb.sheetnames:
        s = wb["048"]
        for r in range(1, s.max_row + 1):
            c1 = clean_str(s.cell(r, 2).value)
            c2 = clean_str(s.cell(r, 3).value)
            c4 = clean_str(s.cell(r, 5).value)
            if c1 and c4 and c4.lower() not in ["pwd", "login"]:
                add_entry(f"JMG 048 - {c1}", "", c2 or "admin", c4, "Pro", "JMG, 048", "")

    # ==========================================
    # 6. Sheet 051 (Alca Torda)
    # ==========================================
    if "051" in wb.sheetnames:
        s = wb["051"]
        for r in range(3, s.max_row + 1):
            mail = clean_str(s.cell(r, 2).value)
            pwd = clean_str(s.cell(r, 3).value) or clean_str(s.cell(r, 9).value)
            app = clean_str(s.cell(r, 3).value)
            if mail and pwd and "@" in mail:
                add_entry(f"Alca Torda - {mail}", "", mail, pwd, "Pro", "Alca Torda, 051", "")

    # ==========================================
    # 7. Sheet 053
    # ==========================================
    if "053" in wb.sheetnames:
        s = wb["053"]
        for r in range(1, s.max_row + 1):
            u = clean_str(s.cell(r, 2).value)
            p = clean_str(s.cell(r, 3).value)
            if u and p and u.lower() != "login":
                add_entry(f"Compte 053 - {u}", "", u, p, "Pro", "053", "")

    # ==========================================
    # 8. Sheet 055 (Lémenu)
    # ==========================================
    if "055" in wb.sheetnames:
        s = wb["055"]
        for r in range(1, s.max_row + 1):
            c1 = clean_str(s.cell(r, 2).value)
            c2 = clean_str(s.cell(r, 3).value)
            c3 = clean_str(s.cell(r, 4).value)
            c4 = clean_str(s.cell(r, 5).value)
            if c1 and c4 and c4.lower() not in ["pwd", "login"]:
                add_entry(f"Lémenu 055 - {c1}", "", c3 or c2 or "admin", c4, "Pro", "Lémenu, 055", "")

    # ==========================================
    # 9. Sheet 057 (Dolibarr & Site)
    # ==========================================
    if "057" in wb.sheetnames:
        s = wb["057"]
        for r in range(1, s.max_row + 1):
            c1 = clean_str(s.cell(r, 2).value)
            c2 = clean_str(s.cell(r, 4).value)
            c3 = clean_str(s.cell(r, 5).value)
            if c2 and c3 and c2.lower() != "login":
                add_entry(f"Projet 057 - {c1 or c2}", "", c2, c3, "Pro", "057, Dolibarr", "")

    # ==========================================
    # 10. Sheet 060
    # ==========================================
    if "060" in wb.sheetnames:
        s = wb["060"]
        for r in range(1, s.max_row + 1):
            u = clean_str(s.cell(r, 2).value)
            p = clean_str(s.cell(r, 3).value)
            if u and p:
                add_entry(f"API 060 - {u}", "", u, p, "Pro", "060, API", "")

    # Écriture dans le fichier CSV final
    os.makedirs(os.path.dirname(CSV_PATH), exist_ok=True)
    with open(CSV_PATH, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=["title", "url", "username", "password", "category", "tags", "notes"])
        writer.writeheader()
        for rec in records:
            writer.writerow(rec)

    print(f"✅ Extraction terminée avec succès : {len(records)} identifiants exportés dans {CSV_PATH}")

if __name__ == "__main__":
    run()
