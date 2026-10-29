#!/usr/bin/env bash
set -e

echo "🔨 Compilation du binaire release..."
cargo build --release

APP_NAME="P2TSecret"
APP_DIR="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "📦 Création du bundle macOS ${APP_DIR}..."
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copie du binaire
cp target/release/p2tsecret "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# Copie de l'icône de l'application
if [ -f "assets/AppIcon.icns" ]; then
    echo "🎨 Intégration de l'icône AppIcon.icns..."
    cp assets/AppIcon.icns "${RESOURCES_DIR}/AppIcon.icns"
fi

# Génération du Info.plist
cat <<EOF > "${CONTENTS_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>fr</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.p2t.secret</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "✅ Application macOS prête : ${APP_DIR}"
echo "💡 Vous pouvez la lancer avec : open \"${APP_DIR}\""
echo "   Ou la copier dans votre dossier /Applications :"
echo "   cp -r \"${APP_DIR}\" /Applications/"
