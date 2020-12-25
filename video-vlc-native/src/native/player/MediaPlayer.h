#ifndef POPCORNPLAYER_MEDIAPLAYER_H
#define POPCORNPLAYER_MEDIAPLAYER_H

#include "Media.h"
#include "MediaPlayerState.h"

#include <Log.h>
#include <QObject>
#include <QtGui/QWidgetSet>
#include <libvlc/vlc/vlc.h>

Q_DECLARE_METATYPE(MediaPlayerState)

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
     */
    void play(Media *media);

    /**
     * Seek the given time in this media player.
     *
     * @param time The time to seek in millis.
     */
    void seek(long time);

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

    /**
     * Set the media player audio volume.
     *
     * @param volume The volume in percents (0 = mute, 100 = 0dB).
     */
    void setVolume(int volume);

    /**
     * Get the current media player state.
     *
     * @return Returns the state of the media player.
     */
    MediaPlayerState state();

    /**
     * Get the current time of the media player.
     * The time is -1 if the media player is being disposed.
     *
     * @return Returns the current time of the media player, or -1 on failure.
     */
    long time();

    /**
     * Get the current duration of the media player.
     * The duration is -1 if no media is playing.
     *
     * @return Returns the duration of the current media playback, or -1 on failure.
     */
    long duration();

public slots:
    /**
     * Set the video surface this media player must render on.
     *
     * @param wid The window handle to use.
     */
    void setVideoSurface(WId wid);

private slots:
    /**
     * Set the media duration of the current media playback.
     *
     * @param newValue The new media duration in milliseconds.
     */
    void setMediaDuration(long newValue);

    /**
     * Invoked when the media is parsed.
     */
    void onMediaParsed();

signals:
    /**
     * Signals that the time has been changed of the media player.
     *
     * @param newValue The new time value of the media player.
     */
    void timeChanged(long newValue);

    /**
     * Signals that the duration has been changed of the current media playback.
     *
     * @param newValue The new duration value of the media playback.
     */
    void durationChanged(long newValue);

    /**
     * Signals that the player state has been changed.
     *
     * @param newState The new player state.
     */
    void stateChanged(MediaPlayerState newState);

private:
    libvlc_instance_t *_vlcInstance;
    libvlc_media_player_t *_vlcMediaPlayer;
    libvlc_media_list_player_t *_vlcMediaList;
    libvlc_event_manager_t *_vlcEventManager;
    Media *_media;
    MediaPlayerState _state = MediaPlayerState::UNKNOWN;
    std::string _subtitleUri;
    std::atomic<long> _seek = -1;
    Log *_log;

    /**
     * Initialize this media player instance.
     */
    void initializeMediaPlayer();

    /**
     * Handle the VLC error that occurred.
     */
    void handleVlcError();

    /**
     * Release this media player instance.
     */
    void releaseMediaPlayer();

    /**
     * Release the media list from this media player instance.
     */
    void releaseMediaList();

    /**
     * Subscribe this media player to the VLC events.
     */
    void subscribeEvents();

    /**
     * Unsubscribe this media player instance from the VLC events.
     */
    void unsubscribeEvents();

    /**
     * Connect the media events to this media player.
     *
     * @param media The media item to connect the events of.
     */
    void subscribeToMediaEvents(Media *media);

    /**
     * Update the current player state.
     *
     * @param newState The new media player state.
     */
    void updateState(MediaPlayerState newState);

    /**
     * Update the active media playback item.
     *
     * @param media The new active media playback item.
     */
    void updateActiveMediaItem(Media *media);

    /**
     * Release the old media item from this media player.
     */
    void releaseMediaItem();

    /**
     * Apply the stored subtitle uri file to the current media playback.
     *
     * @param subtitleUri The subtitle uri to apply.
     */
    void applySubtitleFile(const std::string &subtitleUri);

    /**
     * Apply the given seek value to the current media playback.
     *
     * @param time The new timestamp value to seek.
     */
    void applySeek(long time);

    /**
     * Verify if the given subtitle uri is valid.
     * This will validate if the subtitle uri contains a scheme.
     *
     * @param subtitleUri The subtitle uri to verify.
     * @return Returns true if the given uri is valid, else false.
     */
    static bool isValidSubtitleUri(const std::string &subtitleUri);

    static void vlcCallback(const libvlc_event_t *event, void *instance);

    static QList<libvlc_event_e> eventList();
};

#endif //POPCORNPLAYER_MEDIAPLAYER_H
