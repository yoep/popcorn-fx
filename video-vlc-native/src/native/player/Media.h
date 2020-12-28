#ifndef POPCORNPLAYER_MEDIA_H
#define POPCORNPLAYER_MEDIA_H

#include "../../../../shared/Log.h"
#include "MediaState.h"

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
     * Get the VLC subitems of this media item.
     * The state of the media item should be PARSED before using this method.
     *
     * @return Returns the VLC media subitems.
     */
    libvlc_media_list_t *subitems();

    /**
     * Verify if this media item has sub items.
     * The state of the media item should be PARSED before using this method.
     *
     * @return Returns true if this item has subitems, else false.
     */
    bool hasSubitems();

    /**
     * Get the state of the media item.
     *
     * @return Returns the current state.
     */
    MediaState state();

signals:
    /**
     * Signals that the state of the media item has been changed.
     *
     * @param newState The new state of the media item.
     */
    void stateChanged(MediaState newState);

    /**
     * Signals that the media item has been parsed and ready to use.
     */
    void parsed();

private:
    libvlc_instance_t *_vlcInstance;
    libvlc_event_manager_t *_vlcEvent;
    libvlc_media_t *_vlcMedia;
    MediaState _state = MediaState::UNKNOWN;
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
     * Update the duration of the media item.
     *
     * @param duration The new duration value.
     */
    void updateDuration(long duration);

    /**
     * Update the media state based on the VLC state.
     *
     * @param vlcState The new vlc media state.
     */
    void updateState(int vlcState);

    void onParsedEvent();

    void invokeStateChange(MediaState newState);

    /**
     * Get the total subitems of the media item.
     *
     * @return Returns the total subitems.
     */
    int countSubitems();

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
