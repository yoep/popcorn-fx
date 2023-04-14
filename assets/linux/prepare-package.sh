#!/bin/bash

INSTALLATION_PACKAGE=./target/package

echo "Preparing Linux package"
mkdir "$INSTALLATION_PACKAGE/DEBIAN"
cp -v ./assets/linux/control "$INSTALLATION_PACKAGE/DEBIAN/"
cp -v ./assets/linux/postrm "$INSTALLATION_PACKAGE/DEBIAN/postinst"
cp -v ./assets/linux/postrm "$INSTALLATION_PACKAGE/DEBIAN/postrm"
chmod +x "$INSTALLATION_PACKAGE/DEBIAN/postrm" "$INSTALLATION_PACKAGE/DEBIAN/postinst"

mkdir -p "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v ./assets/linux/popcorn-time "$INSTALLATION_PACKAGE/opt/popcorn-time/"
cp -v -R "$INSTALLATION_PACKAGE/runtimes" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/"
cp -v "$INSTALLATION_PACKAGE/libpopcorn_fx.so" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v "$INSTALLATION_PACKAGE/popcorn-time.jar" "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"
cp -v ./assets/linux/libjlibtorrent.so "$INSTALLATION_PACKAGE/opt/popcorn-time/main/${VERSION}/"

mkdir -p "$INSTALLATION_PACKAGE/usr/share/applications"
cp -v ./assets/linux/popcorn-time.dekstop "$INSTALLATION_PACKAGE/usr/share/applications/"