#ifndef POPCORNPLAYER_MEDIA_H
#define POPCORNPLAYER_MEDIA_H

#include <Log.h>
#include <QList>
#include <QObject>
#include <QtCore/QArgument>
#include <libvlc/vlc/vlc.h>

class Media : public QObject {
    Q_OBJECT

public:
    Media(const char *mrl, libvlc_instance_t *vlcInstance);

    ~Media();

    /**
     * Get the VLC media instance of this media item.
     * If this media instance failed to initialize the MRL, the return vlc instance will be NULL.
     *
     * @return Returns the VLC media item instance of this media.
     */
    libvlc_media_t *vlcMedia();

    /**
     * Get the duration of the media item.
     *
     * @return Returns the duration of the media item in milliseconds.
     */
    long getDuration();

signals:
    /**
     * Signal that the duration of the media item has been changed.
     *
     * @param newValue The new duration value.
     */
    void durationChanged(long newValue);

private:
    libvlc_instance_t *_vlcInstance;
    libvlc_event_manager_t *_vlcEvent;
    libvlc_media_t *_vlcMedia;
    Log *_log;
    const char *_mrl;

    /**
     * Initialize this media instance based on the MRL.
     */
    void initializeMedia();

    /**
     * Create the VLC media item from the given path.
     *
     * @param path The file path to create the VLC media item from.
     * @returns Returns the created VLC media item.
     */
    libvlc_media_t *createFromFile(const char *path);

    /**
     * Create the VLC media item from the given url.
     *
     * @param url The url to create the VLC media item from.
     * @returns Returns the created VLC media item.
     */
    libvlc_media_t *createFromUrl(const char *url);

    /**
     * Subscribe to the VLC events.
     */
    void subscribeEvents();

    /**
     * Unsubscribe from the VLC events.
     */
    void unsubscribeEvents();

    /**
     * Verify if the given mrl is a HTTP url.
     *
     * @param mrl The mrl to verify.
     * @return Returns true if the MRL is a HTTP url, else false.
     */
    static bool isHttpUrl(const char *mrl);

    /**
     * The VLC callback method for the Media instance.
     *
     * @param event The VLC media event that was triggered.
     * @param instance The Media instance for which the event was triggered.
     */
    static void vlcCallback(const libvlc_event_t *event, void *instance);

    /**
     * Get the VLC events list for the Media.
     *
     * @return Returns the list of events the Media needs to subscribe to.
     */
    static QList<libvlc_event_e> eventList();
};

#endif //POPCORNPLAYER_MEDIA_H
