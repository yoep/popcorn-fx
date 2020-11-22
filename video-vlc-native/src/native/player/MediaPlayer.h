#ifndef POPCORNPLAYER_MEDIAPLAYER_H
#define POPCORNPLAYER_MEDIAPLAYER_H

#include "Media.h"

#include <Log.h>
#include <QObject>
#include <QtGui/QWidgetSet>
#include <libvlc/vlc/vlc.h>

class MediaPlayer : public QObject {
    Q_OBJECT

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
     * Play the given media item in this media player instance.
     *
     * @param media The media item to play.
     * @return Returns true if the media playback was started with success, else false.
     */
    bool play(Media *media);

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

signals:
    /**
     * Signals that the time has been changed of the media player.
     *
     * @param newValue The new time value of the media player.
     */
    void timeChanged(long newValue);

private:
    libvlc_instance_t *_vlcInstance;
    libvlc_media_player_t *_vlcMediaPlayer;
    libvlc_event_manager_t *_vlcEventManager;
    Media *_media;
    Log *_log;

    /**
     * Initialize this media player instance.
     */
    void initializeMediaPlayer();

    /**
     * Handle the VLC error that occurred.
     */
    void handleVlcError();

    void releaseMediaPlayerIfNeeded();

    void subscribeEvents();

    void unsubscribeEvents();

    static void vlcCallback(const libvlc_event_t *event, void *instance);

    static QList<libvlc_event_e> eventList();
};

#endif //POPCORNPLAYER_MEDIAPLAYER_H
