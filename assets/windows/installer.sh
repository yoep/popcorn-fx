#!/bin/bash

cp -v assets/windows/ffprobe.exe target/package/

echo "Building EXE installer"
powershell.exe -Command "iscc.exe /Otarget/ /Fpopcorn-time_${VERSION} \"./assets/windows/installer.iss\""