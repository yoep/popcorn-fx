#!/bin/bash

IMAGE_NAME="popcorn-time_${VERSION}.dmg"
OUTPUT_TARGET=./target/${IMAGE_NAME}
VOLUME_LOCATION=/Volumes/${IMAGE_NAME}

detach_volume() {
  echo "Detaching image ${IMAGE_NAME}"
  hdiutil detach ${VOLUME_LOCATION}
}

echo "Creating DMG image ${IMAGE_NAME}"
hdiutil create -srcfolder ./target/package -volname "${IMAGE_NAME}" -ov -fs "HFS+" -format "UDRW" ${OUTPUT_TARGET}

if [ -d "/Volumes/${IMAGE_NAME}" ]; then
  detach_volume
fi

echo "Attaching DMG image ${IMAGE_NAME} to ${VOLUME_LOCATION}"
hdiutil attach -readwrite -noverify -noautoopen ${OUTPUT_TARGET} &
sleep 2000

echo "Creating symlink to Applications"
ln -s "${VOLUME_LOCATION}/Applications" "/Applications"
echo "Executing applescript"
osascript ./assets/mac/dmg.applescript "${IMAGE_NAME}"

echo "Updating DMG permissions"
chmod -Rf u+r,go-w ${VOLUME_LOCATION}
SetFile -a -C ${VOLUME_LOCATION}
detach_volume
