#include <QtWidgets/QWidget>
#include <QtWidgets/QFrame>
#include "vlc/vlc.h"

#ifndef POPCORNDESKTOPPLAYER_PLAYER_H
#define POPCORNDESKTOPPLAYER_PLAYER_H

class VideoPlayer : public QFrame {
public:
    VideoPlayer();

    ~VideoPlayer() override;

    void playUrl(char const *url);

    void playFile(char const *path);

    void pause();

    void resume();

    void stop();

private:
    libvlc_instance_t *vlcInstance;
    libvlc_media_player_t *mediaPlayer;
    libvlc_media_t *media;

    void play();

    static void handleVlcError();
};


#endif //POPCORNDESKTOPPLAYER_PLAYER_H
