#include "MediaPlayerFactory.h"

#include "MediaPlayer.h"

#include <Log.h>
#include <libvlc/vlc/vlc.h>

MediaPlayerFactory *MediaPlayerFactory::instance = nullptr;

//region Constructors

MediaPlayerFactory::MediaPlayerFactory()
{
    this->log = Log::getInstance();
}

//endregion

//region Methods

MediaPlayer *MediaPlayerFactory::create()
{
    MediaPlayerFactory *factory = getInstance();

    // initialize VLC args
    factory->log->trace("Creating new media player");
    const char *vlcArgs = factory->log->getLevel() & TRACE_FLAG ? "--verbose=2" : nullptr;
    int argc = factory->log->getLevel() & TRACE_FLAG ? 1 : 0;
    const char *const *argv = &vlcArgs;

    // create a new vlc instance
    factory->log->trace("Initializing new VLC instance");
    auto *vlcInstance = libvlc_new(argc, argv);

    // check if a vlc instance was created with success
    // if not, show an error dialog
    if (vlcInstance == nullptr) {
        factory->log->error("Failed to initialize new VLC instance");
        return nullptr;
    }

    auto *player = new MediaPlayer(vlcInstance);
    factory->log->debug("Media player created");
    return player;
}

//endregion

//region Functions

MediaPlayerFactory *MediaPlayerFactory::getInstance()
{
    if (!instance) {
        instance = new MediaPlayerFactory();
    }

    return instance;
}

//endregion
