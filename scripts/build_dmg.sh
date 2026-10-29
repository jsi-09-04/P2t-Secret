#!/usr/bin/env bash
set -e

# S'assurer que le .app est généré
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"${DIR}/build_app.sh"

APP_NAME="P2TSecret"
DMG_NAME="${APP_NAME}.dmg"
RELEASE_DIR="target/release"
APP_PATH="${RELEASE_DIR}/${APP_NAME}.app"
DMG_PATH="${RELEASE_DIR}/${DMG_NAME}"
STAGING_DIR="${RELEASE_DIR}/dmg_staging"

echo "💿 Préparation de l'image disque .dmg pour ${APP_NAME}..."
rm -rf "${STAGING_DIR}" "${DMG_PATH}"
mkdir -p "${STAGING_DIR}"

# Copie de l'application
cp -R "${APP_PATH}" "${STAGING_DIR}/"

# Création du lien symbolique vers /Applications pour le glisser-déposer
ln -s /Applications "${STAGING_DIR}/Applications"

# Intégration de l'icône de volume du DMG
if [ -f "assets/AppIcon.icns" ]; then
    echo "🎨 Application de l'icône au volume du DMG..."
    cp assets/AppIcon.icns "${STAGING_DIR}/.VolumeIcon.icns"
    if command -v SetFile >/dev/null 2>&1; then
        SetFile -c icnC "${STAGING_DIR}/.VolumeIcon.icns" || true
        SetFile -a C "${STAGING_DIR}" || true
    fi
fi

# Création du fichier DMG compressé
echo "🗜️  Création du DMG avec hdiutil..."
hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${STAGING_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_PATH}"

# Nettoyage du staging
rm -rf "${STAGING_DIR}"

echo "🎉 Fichier .dmg créé avec succès !"
echo "📍 Emplacement : ${DMG_PATH}"
echo "💡 Vous pouvez l'ouvrir avec : open \"${DMG_PATH}\""
