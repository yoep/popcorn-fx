#include "MediaPlayer.h"

#include <QtGui/QWidgetSet>
#include <regex>
#include <string>

using namespace std;

//region Constructors

MediaPlayer::MediaPlayer(libvlc_instance_t *vlcInstance)
{
    this->log = Log::getInstance();
    this->vlcInstance = vlcInstance;
    this->vlcMediaPlayer = libvlc_media_player_new(vlcInstance);
    this->media = nullptr;

    libvlc_media_player_retain(vlcMediaPlayer);
}

MediaPlayer::~MediaPlayer()
{
    releaseMediaPlayerIfNeeded();
}

//endregion

//region Methods

bool MediaPlayer::play(const char *mrl)
{
    if (mrl == nullptr) {
        log->warn("Unable to play empty MRL, ignoring media player play");
        return false;
    }

    if (isHttpUrl(mrl)) {
        return playUrl(mrl);
    } else {
        return playFile(mrl);
    }
}

void MediaPlayer::pause()
{
    try {
        log->info("Pausing media player");
        libvlc_media_player_set_pause(vlcMediaPlayer, 1);
    } catch (std::exception &ex) {
        log->error("An error occurred while pausing the media playback", ex);
    }
}

void MediaPlayer::resume()
{
    try {
        log->info("Resuming media player");
        libvlc_media_player_set_pause(vlcMediaPlayer, 0);
    } catch (std::exception &ex) {
        log->error("An error occurred while resuming the media playback", ex);
    }
}

void MediaPlayer::stop()
{
    try {
        log->info("Stopping media player");
        libvlc_media_player_stop(vlcMediaPlayer);
    } catch (std::exception &ex) {
        log->error("An error occurred while resuming the media playback", ex);
    }
}

void MediaPlayer::setVideoSurface(WId wid)
{
#if defined(Q_OS_WIN)
    log->trace("Adding Windows window to the VLC media player");
    void *drawable = (void *)wid;
    libvlc_media_player_set_hwnd(vlcMediaPlayer, drawable);
#elif defined(Q_OS_MAC)
    log->trace("Adding Mac window to the VLC media player");
    void *drawable = (void *)wid;
    libvlc_media_player_set_nsobject(vlcMediaPlayer, drawable);
#else
    log->trace("Adding X window to the VLC media player");
    libvlc_media_player_set_xwindow(vlcMediaPlayer, wid);
#endif
}

void MediaPlayer::setSubtitleFile(const char *uri)
{
    log->debug(std::string("Adding new subtitle track: ") + uri);

    if (libvlc_media_player_add_slave(vlcMediaPlayer, libvlc_media_slave_type_subtitle, uri, true) == 0) {
        log->info(std::string("Subtitle track \"") + uri + "\" has been added with success");
    } else {
        log->error(std::string("Failed to add subtitle track ") + uri);
    }
}

void MediaPlayer::setSubtitleDelay(long delay)
{
    log->debug(std::string("Updating subtitle delay to ") + std::to_string(delay) + "ms");
    libvlc_video_set_spu_delay(vlcMediaPlayer, delay);
}

//endregion

//region Functions

bool MediaPlayer::playFile(const char *path)
{
    log->debug(std::string("Creating media for file path: ") + path);
    media = libvlc_media_new_path(vlcInstance, path);

    if (media == nullptr) {
        log->warn(std::string("Unable to create media for path ") + path);
        return false;
    }

    log->info(std::string("Playing file path: ") + path);
    return play();
}
bool MediaPlayer::playUrl(const char *url)
{
    log->debug(std::string("Creating media for url: ") + url);
    media = libvlc_media_new_location(vlcInstance, url);

    if (media == nullptr) {
        log->warn(std::string("Unable to create media for url ") + url);
        return false;
    }

    log->info(std::string("Playing url: ") + url);
    return play();
}

bool MediaPlayer::play()
{
    libvlc_media_player_set_media(vlcMediaPlayer, media);

    int result = libvlc_media_player_play(vlcMediaPlayer);

    if (result == -1) {
        handleVlcError();
        return false;
    } else {
        return true;
    }
}

bool MediaPlayer::isHttpUrl(const char *url)
{
    std::string value = url;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}

void MediaPlayer::releaseMediaPlayerIfNeeded()
{
    if (vlcMediaPlayer == nullptr) {
        return;
    }

    log->trace("Releasing current VLC media player resources");
    // stop the current media playback in case any media is still playing
    stop();
    // release the media player which was retained during construction if this media player
    libvlc_media_player_release(vlcMediaPlayer);
}

void MediaPlayer::handleVlcError()
{
    const char *message = libvlc_errmsg();

    if (message != nullptr) {
        log->error(std::string("Media player encountered a VLC error: ") + message);
    }
}

//endregion
