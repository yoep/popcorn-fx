#!/bin/bash

# switch working directory
cd "/opt/popcorn-time" || return

export OPENJFX=/opt/openjfx/13/lib/
export PATH=/opt/popcorn-time/:${OPENJFX}:/usr/lib/arm-linux-gnueabihf:${PATH}
export DISPLAY=:0

retries=0

start() {
  java \
    -Dvlc.path=/usr/lib/arm-linux-gnueabihf \
    -Djava.library.path="${PATH}" \
    -Djavafx.platform=gtk \
    -Dprism.verbose=true \
    -Dprism.lcdtext=false \
    -Dprism.dirtyopts=false \
    -Dsun.awt.disablegrab=true \
    -XX:+UseG1GC \
    -Xmx400M \
    -Xms100M \
    -p "${OPENJFX}" \
    --add-modules javafx.controls,javafx.fxml,javafx.graphics \
    -jar /opt/popcorn-time/popcorn-time.jar \
    --tv --disable-keep-alive --maximized --disable-youtube-video-player --disable-javafx-video-player ${@} 2>&1 | tee start.log

  status=${PIPESTATUS[0]}
  echo "Exited with status ${status}"
  return ${status}
}

start ${@}
exitStatus=$?

# this is a workaround created for the libpango crashes which often occur during startup of the application
# it's a dirty one, but the only way to make sure the application is able to correctly start at some point
while [[ ${exitStatus} != 0 ]]; do
  retries=$((retries + 1))

  # if we already tried 3 times
  # reboot the device as it will otherwise get stuck
  if [[ ${retries} == 3 ]]; then
    sudo reboot
  fi

  start ${@}
  exitStatus=$?
done

exit 0
