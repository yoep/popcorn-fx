#ifndef POPCORNPLAYER_MEDIAPLAYERFACTORY_H
#define POPCORNPLAYER_MEDIAPLAYERFACTORY_H

#include "../../../../shared/Log.h"
#include "MediaPlayer.h"

class MediaPlayerFactory {
public:
    virtual ~MediaPlayerFactory();

    /**
     * Create a new media player instance.
     *
     * @return Returns the new media player.
     */
    static MediaPlayer *createPlayer();

    /**
     * Dispose the MediaPlayerFactory resources.
     */
    static void dispose();

    /**
     * Create a new media instance for the given MRL.
     *
     * @param mrl The MRL to create a media instance for.
     * @return Returns the created media instance.
     */
    static Media *createMedia(const char *mrl);

private:
    MediaPlayerFactory();

    libvlc_instance_t *_vlcInstance;
    Log *_log;

    /**
     * Get the VLC instance of this media factory.
     *
     * @return Returns the VLC instance.
     */
    libvlc_instance_t * getVlcInstance();

    static MediaPlayerFactory *instance;

    static MediaPlayerFactory *getInstance();
};

#endif //POPCORNPLAYER_MEDIAPLAYERFACTORY_H
