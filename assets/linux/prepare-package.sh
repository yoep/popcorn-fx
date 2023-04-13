#!/bin/bash

echo "Preparing Linux package"
mkdir ./target/package/DEBIAN
cp -v ./assets/linux/control ./target/package/DEBIAN/
cp -v ./assets/linux/postrm ./target/package/DEBIAN/postrm
chmod +x ./target/package/DEBIAN/postrm

mkdir -p ./target/package/opt/popcorn-time
cp -v ./assets/linux/popcorn-time ./target/package/opt/popcorn-time/
cp -v -R ./target/package/runtimes "./target/package/opt/popcorn-time/main/"
cp -v ./target/package/libpopcorn_fx.so "./target/package/opt/popcorn-time/main/${VERSION}/"
cp -v ./target/package/popcorn-time.jar "./target/package/opt/popcorn-time/main/${VERSION}/"
cp -v ./assets/linux/libjlibtorrent.so "./target/package/opt/popcorn-time/main/${VERSION}/"

mkdir -p ./target/package/usr/share/applications
cp -v ./assets/linux/popcorn-time.dekstop ./target/package/usr/share/applications/