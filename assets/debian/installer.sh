#!/bin/bash

PACKAGE_DIR=./target/package
INSTALLATION_PACKAGE=./target/package/installer
ASSETS_DIR=./assets/debian

if [[ -v "${VERSION}" ]]; then
  echo "VERSION has not been set";
  exit 1;
fi
if [[ ! -d "${ASSETS_DIR}" ]]; then
  echo "Assets directory '${ASSETS_DIR}' does not exist";
  exit 1;
fi

echo "Preparing Linux package"
mkdir -p "$INSTALLATION_PACKAGE/DEBIAN"
cp -v ${ASSETS_DIR}/control "$INSTALLATION_PACKAGE/DEBIAN/"
cp -v ${ASSETS_DIR}/postinst "$INSTALLATION_PACKAGE/DEBIAN/postinst"
cp -v ${ASSETS_DIR}/postrm "$INSTALLATION_PACKAGE/DEBIAN/postrm"
chmod +x "$INSTALLATION_PACKAGE/DEBIAN/postrm" "$INSTALLATION_PACKAGE/DEBIAN/postinst"

mkdir -p "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v ${ASSETS_DIR}/popcorn-time "$INSTALLATION_PACKAGE/opt/popcorn-time/"
cp -v ${ASSETS_DIR}/popcorn-time.png "$INSTALLATION_PACKAGE/opt/popcorn-time/"
cp -v -R "$PACKAGE_DIR/runtimes" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/"
cp -v "$PACKAGE_DIR/libpopcorn_fx.so" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v "$PACKAGE_DIR/popcorn-time.jar" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v ${ASSETS_DIR}/libjlibtorrent.so "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
chmod 644 "$INSTALLATION_PACKAGE/opt/popcorn-time/popcorn-time.png"

mkdir -p "$INSTALLATION_PACKAGE/usr/share/applications"
cp -v ${ASSETS_DIR}/popcorn-time.desktop "$INSTALLATION_PACKAGE/usr/share/applications/"
chmod 644 "$INSTALLATION_PACKAGE/usr/share/applications/popcorn-time.desktop"

echo "Building DEB package"
dpkg-deb --build -Zgzip target/package/installer target/popcorn-time_${VERSION}.deb

echo "Cleaning installer"
rm -rf target/package/installer