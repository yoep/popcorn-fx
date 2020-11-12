#include "Log.h"
#include "vlc/vlc.h"
#include <QtWidgets/QFrame>
#include <QtWidgets/QWidget>

#ifndef POPCORN_PLAYER_PLAYER_H
#define POPCORN_PLAYER_PLAYER_H

class VideoPlayer : public QWidget {
public:
    explicit VideoPlayer(QWidget* parent = nullptr);

    ~VideoPlayer() override;

    void playUrl(char const* url);

    void playFile(char const* path);

    void pause();

    void resume();

    void stop();

private:
    libvlc_instance_t* vlcInstance{};
    libvlc_media_player_t* mediaPlayer{};
    libvlc_media_t* media{};
    Log* log;

    void play();

    void initializeUi();

    void initializeVlc();

    void handleVlcError();
    void releaseMediaPlayerIfNeeded();
};

#endif //POPCORN_PLAYER_PLAYER_H
