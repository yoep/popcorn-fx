#ifndef POPCORNPLAYER_MEDIAPLAYER_H
#define POPCORNPLAYER_MEDIAPLAYER_H

#include <Log.h>
#include <QtGui/QWidgetSet>
#include <libvlc/vlc/vlc.h>

class MediaPlayer {
public:
    /**
     * Create a new media player instance.
     *
     * @param vlcInstance The VLC instance used by the media player.
     */
    explicit MediaPlayer(libvlc_instance_t *vlcInstance);

    /**
     * Destroy the media player instance.
     * This will release the VLC resources used by this media player.
     */
    ~MediaPlayer();

    /**
     * Play the given MRL within the media player.
     *
     * @param mrl The MRL to play.
     * @return Returns true if the MRL playback was started, else false.
     */
    bool play(const char *mrl);

    /**
     * Pause the current media playback.
     */
    void pause();

    /**
     * Resume the current media playback.
     */
    void resume();

    /**
     * Stop the current media playback.
     */
    void stop();

    /**
     * Set the video surface this media player must render on.
     *
     * @param wid The window handle to use.
     */
    void setVideoSurface(WId wid);

    /**
     * Set the subtitle file for the current playback.
     *
     * @param uri The absolute uri path to the subtitle file.
     */
    void setSubtitleFile(const char *uri);

    /**
     * Set the subtitle delay for the current subtitle file (if one is set).
     *
     * @param delay The delay in microseconds.
     */
    void setSubtitleDelay(long delay);

private:
    libvlc_instance_t *vlcInstance;
    libvlc_media_player_t *vlcMediaPlayer;
    libvlc_media_t *media;
    Log *log;

    bool playFile(const char *path);

    bool playUrl(const char *url);

    bool play();

    void handleVlcError();

    void releaseMediaPlayerIfNeeded();

    static bool isHttpUrl(const char *url);
};

#endif //POPCORNPLAYER_MEDIAPLAYER_H
