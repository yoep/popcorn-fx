#include <QtWidgets/QWidget>
#include <QtWidgets/QFrame>
#include "vlc/vlc.h"

#ifndef POPCORN_PLAYER_PLAYER_H
#define POPCORN_PLAYER_PLAYER_H

class VideoPlayer : public QWidget {
public:
    explicit VideoPlayer(QWidget* parent = nullptr);

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

    void initializeUi();

    static void handleVlcError();
};


#endif //POPCORN_PLAYER_PLAYER_H
