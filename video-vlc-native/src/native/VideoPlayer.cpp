#include "VideoPlayer.h"
#include <iostream>
#include <QtWidgets/QStackedLayout>

using namespace std;

VideoPlayer::VideoPlayer() : QWidget() {
    cout << "Initializing player" << endl;
    videoWidget = new QFrame(this);

    vlcInstance = libvlc_new(0, nullptr);
    mediaPlayer = nullptr;
    media = nullptr;

    initializeLayout();
}

VideoPlayer::~VideoPlayer() {
    stop();
    libvlc_release(vlcInstance);
}

void VideoPlayer::playFile(char const *path) {
    cout << std::string("Playing file path: ") + path << endl;
    media = libvlc_media_new_path(vlcInstance, path);

    // start the playback of the media
    play();
}

void VideoPlayer::playUrl(char const *url) {
    cout << std::string("Playing url: ") + url << endl;
    media = libvlc_media_new_location(vlcInstance, url);

    // start the playback of the media
    play();
}

void VideoPlayer::initializeLayout() {
    auto *layout = new QStackedLayout();
    layout->addWidget(videoWidget);
    setLayout(layout);
}

void VideoPlayer::play() {
    // check if a media is present
    // if not, raise an exception that the play was called to early
    if (media == nullptr) {
        throw exception();
    }

    // create a new media player for the current media
    mediaPlayer = libvlc_media_player_new_from_media(media);

    void *drawable = (void *) videoWidget->winId();

#if defined(Q_OS_WIN)
    libvlc_media_player_set_hwnd(mediaPlayer, drawable);
#elif defined(Q_OS_MAC)
    libvlc_media_player_set_agl(mediaPlayer, drawable);
#else
    libvlc_media_player_set_xwindow(mediaPlayer, drawable);
#endif

    if (libvlc_media_player_play(mediaPlayer) == -1) {
        handleVlcError();
    }
}

void VideoPlayer::pause() {
    libvlc_media_player_set_pause(mediaPlayer, 1);
}

void VideoPlayer::resume() {
    libvlc_media_player_set_pause(mediaPlayer, 0);
}

void VideoPlayer::stop() {
    if (mediaPlayer != nullptr) {
        cout << "Stopping current media player" << endl;
        libvlc_media_player_stop(mediaPlayer);
    }

    if (media != nullptr) {
        libvlc_media_release(media);
    }

    mediaPlayer = nullptr;
    media = nullptr;
}

void VideoPlayer::handleVlcError() {
    const char *message = libvlc_errmsg();

    if (message != nullptr) {
        cerr << std::string("A VLC error occurred: ") + message << endl;
    }
}
