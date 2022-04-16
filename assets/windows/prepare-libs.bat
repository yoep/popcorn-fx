echo "Extracting libraries"
7z x ffprobe.7z -aoa
7z x jlibtorrent.7z -aoa

echo "Extracting VLC libraries"
cd ./vlc

7z x vlc.rar plugins lua -aoa