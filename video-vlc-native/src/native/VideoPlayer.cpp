#include "VideoPlayer.h"
#include "AppProperties.h"
#include <iostream>
#include <QtWidgets/QStackedLayout>
#include <QtWidgets/QMessageBox>

using namespace std;

VideoPlayer::VideoPlayer(QWidget *parent) : QWidget(parent) {
    cout << "Initializing video player" << endl;
    vlcInstance = libvlc_new(0, nullptr);

    if (vlcInstance == nullptr) {
        QMessageBox::critical(this, ApplicationTitle, "Failed to initialize libVLC");
    }

    mediaPlayer = nullptr;
    media = nullptr;

    initializeUi();
    cout << "Video player initialized" << endl;
}

VideoPlayer::~VideoPlayer() {
    stop();
    libvlc_release(vlcInstance);
}

void VideoPlayer::initializeUi() {
    // set the initial background of the video player
    this->setStyleSheet("background-color:black;");
    this->setMinimumSize(800, 600);
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

void VideoPlayer::play() {
    // check if a media is present
    // if not, raise an exception that the play was called to early
    if (media == nullptr) {
        throw exception();
    }

    // create a new media player for the current media
    mediaPlayer = libvlc_media_player_new_from_media(media);
    libvlc_media_player_retain(mediaPlayer);

#if defined(Q_OS_WIN)
    void *drawable = (void *) this->winId();
    libvlc_media_player_set_hwnd(mediaPlayer, drawable);
#elif defined(Q_OS_MAC)
    libvlc_media_player_set_agl(mediaPlayer, this->winId());
#else
    libvlc_media_player_set_xwindow(mediaPlayer, this->winId());
#endif

    if (libvlc_media_player_play(mediaPlayer) == -1) {
        handleVlcError();
    }
}

void VideoPlayer::pause() {
    cout << "Pausing media player" << endl;
    libvlc_media_player_set_pause(mediaPlayer, 1);
}

void VideoPlayer::resume() {
    cout << "Resuming media player" << endl;
    libvlc_media_player_set_pause(mediaPlayer, 0);
}

void VideoPlayer::stop() {
    if (mediaPlayer != nullptr) {
        cout << "Stopping current media player" << endl;
        libvlc_media_player_stop(mediaPlayer);
        libvlc_media_player_release(mediaPlayer);
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
        cerr << "A VLC error occurred: " << message << endl;
    }
}
