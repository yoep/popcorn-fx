#!/bin/bash

IMAGE_NAME="popcorn-time_${VERSION}.dmg"
SOURCE_FOLDER=./target/popcorn-time.app
SOURCE_CONTENTS_FOLDER=${SOURCE_FOLDER}/Contents
TMP_OUTPUT_TARGET=./target/tmp_${IMAGE_NAME}
OUTPUT_TARGET=./target/${IMAGE_NAME}
VOLUME_LOCATION=/Volumes/${IMAGE_NAME}

detach_volume() {
  echo "Detaching image ${IMAGE_NAME}"
  hdiutil detach ${VOLUME_LOCATION}
}

echo "Creating Application contents"
rm -rf ${SOURCE_CONTENTS_FOLDER}
mkdir -vp ${SOURCE_CONTENTS_FOLDER}
mkdir -vp ${SOURCE_CONTENTS_FOLDER}/MacOS
mkdir -vp ${SOURCE_CONTENTS_FOLDER}/Plugins
mkdir -vp ${SOURCE_CONTENTS_FOLDER}/Resources/main
mkdir -vp ${SOURCE_CONTENTS_FOLDER}/Resources/main/${VERSION}

envsubst < ./assets/mac/Info.plist > ${SOURCE_CONTENTS_FOLDER}/Info.plist

cp -v ./LICENSE ${SOURCE_CONTENTS_FOLDER}/Resources/
cp -v ./assets/mac/popcorn-time.icns ${SOURCE_CONTENTS_FOLDER}/Resources/
cp -v ./target/package/*.dylib ${SOURCE_CONTENTS_FOLDER}/Resources/
cp -rv ./target/package/runtimes ${SOURCE_CONTENTS_FOLDER}/Resources/main/
cp -rv ./target/package/*.dylib ${SOURCE_CONTENTS_FOLDER}/Resources/main/${VERSION}/
cp -v ./target/package/popcorn-time.jar ${SOURCE_CONTENTS_FOLDER}/Resources/main/${VERSION}/

cp -v ./target/package/popcorn-time ${SOURCE_CONTENTS_FOLDER}/MacOS/

echo "Copying background image"
mkdir -vp ${SOURCE_FOLDER}/.background
cp -v ./assets/mac/background.png ${SOURCE_FOLDER}/.background/background.png

echo "Copying icon file"
cp -v ./assets/mac/popcorn-time.icns ${SOURCE_FOLDER}/.VolumeIcon.icns

echo "Creating DMG image ${IMAGE_NAME}"
hdiutil create -srcfolder ${SOURCE_FOLDER} -volname "${IMAGE_NAME}" -ov -fs "HFS+" -format "UDRW" ${TMP_OUTPUT_TARGET}

if [ -d "/Volumes/${IMAGE_NAME}" ]; then
  detach_volume
fi

echo "Attaching DMG image ${IMAGE_NAME} to ${VOLUME_LOCATION}"
hdiutil attach -readwrite -noverify -noautoopen ${TMP_OUTPUT_TARGET} &
sleep 2

echo "Creating symlink to Applications"
ln -s "/Applications" "${VOLUME_LOCATION}/Applications"
echo "Executing applescript"
osascript ./assets/mac/dmg.applescript "${IMAGE_NAME}"

echo "Updating DMG permissions"
chmod -Rf u+r,go-w ${VOLUME_LOCATION}
SetFile -a -C ${VOLUME_LOCATION}
detach_volume

echo "Compressing image to ${OUTPUT_TARGET}"
hdiutil convert ${TMP_OUTPUT_TARGET} -ov -format UDZO -imagekey -zlib-level=9 -o ${OUTPUT_TARGET}
rm ${TMP_OUTPUT_TARGET}