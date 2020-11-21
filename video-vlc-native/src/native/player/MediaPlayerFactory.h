#ifndef POPCORNPLAYER_MEDIAPLAYERFACTORY_H
#define POPCORNPLAYER_MEDIAPLAYERFACTORY_H

#include "MediaPlayer.h"
#include <Log.h>

class MediaPlayerFactory {
public:
    /**
     * Create a new media player instance.
     *
     * @return Returns the new media player.
     */
    static MediaPlayer *create();

private:
    MediaPlayerFactory();
    Log *log;

    static MediaPlayerFactory *instance;

    static MediaPlayerFactory *getInstance();
};

#endif //POPCORNPLAYER_MEDIAPLAYERFACTORY_H
