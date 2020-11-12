#include "VideoPlayer.h"
#include "AppProperties.h"
#include "Log.h"
#include <QtWidgets/QMessageBox>
#include <QtWidgets/QStackedLayout>

using namespace std;

VideoPlayer::VideoPlayer(QWidget* parent)
    : QWidget(parent)
{
    this->log = Log::getInstance();

    log->trace("Initializing video player");
    initializeVlc();
    initializeUi();
    log->debug("Video player initialized");
}

VideoPlayer::~VideoPlayer()
{
    log->debug("Releasing video player resources");
    stop();
    log->trace("Releasing VLC resources");
    libvlc_release(vlcInstance);
}

void VideoPlayer::initializeUi()
{
    // set the initial background of the video player
    this->setStyleSheet("background-color:red;");
}

void VideoPlayer::initializeVlc()
{
    vlcInstance = libvlc_new(0, nullptr);

    if (vlcInstance == nullptr) {
        QMessageBox::critical(this, ApplicationTitle, "Failed to initialize libVLC");
    }

    mediaPlayer = nullptr;
    media = nullptr;
}

void VideoPlayer::playFile(char const* path)
{
    log->info(std::string("Playing file path: ") + path);
    media = libvlc_media_new_path(vlcInstance, path);

    // start the playback of the media
    play();
}

void VideoPlayer::playUrl(char const* url)
{
    log->info(std::string("Playing url: ") + url);
    media = libvlc_media_new_location(vlcInstance, url);

    // start the playback of the media
    play();
}

void VideoPlayer::play()
{
    // check if a media is present
    // if not, raise an exception that the play was called to early
    if (media == nullptr) {
        throw exception();
    }

    // create a new media player for the current media
    log->trace("Creating media player from media");
    mediaPlayer = libvlc_media_player_new_from_media(media);
    libvlc_media_player_retain(mediaPlayer);

#if defined(Q_OS_WIN)
    void* drawable = (void*)this->winId();
    libvlc_media_player_set_hwnd(mediaPlayer, drawable);
#elif defined(Q_OS_MAC)
    libvlc_media_player_set_agl(mediaPlayer, this->winId());
#else
    log->trace("Adding X window to the VLC media player");
    WId drawable = this->winId();
    libvlc_media_player_set_xwindow(mediaPlayer, drawable);
#endif

    if (libvlc_media_player_play(mediaPlayer) == -1) {
        log->error("VLC media playback failed to start");
        handleVlcError();
    }
}

void VideoPlayer::pause()
{
    if (mediaPlayer == nullptr) {
        log->debug("Ignoring pause action as no media player is currently present");
        return;
    }

    try {
        log->info("Pausing media player");
        libvlc_media_player_set_pause(mediaPlayer, 1);
    } catch (std::exception& ex) {
        log->error("An error occurred while pausing the media playback", ex);
    }
}

void VideoPlayer::resume()
{
    if (mediaPlayer == nullptr) {
        log->debug("Ignoring pause action as no media player is currently present");
        return;
    }

    try {
        log->info("Resuming media player");
        libvlc_media_player_set_pause(mediaPlayer, 0);
    } catch (std::exception& ex) {
        log->error("An error occurred while resuming the media playback", ex);
    }
}

void VideoPlayer::stop()
{
    log->info("Stopping current media player");

    releaseMediaPlayerIfNeeded();

    if (media != nullptr) {
        log->trace("Releasing VLC media resources");
        libvlc_media_release(media);
    }

    mediaPlayer = nullptr;
    media = nullptr;
}
void VideoPlayer::releaseMediaPlayerIfNeeded()
{
    if (mediaPlayer == nullptr) {
        return;
    }

    log->trace("Releasing current VLC media player resources");
    libvlc_media_player_stop(mediaPlayer);
    libvlc_media_player_release(mediaPlayer);
}

void VideoPlayer::handleVlcError()
{
    const char* message = libvlc_errmsg();

    if (message != nullptr) {
        log->error(std::string("A VLC error occurred: ") + message);
    }
}
