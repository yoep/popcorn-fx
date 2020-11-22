#include "MediaPlayerFactory.h"

#include "MediaPlayer.h"

#include <Log.h>
#include <LogLevelFlags.h>
#include <libvlc/vlc/vlc.h>

MediaPlayerFactory *MediaPlayerFactory::instance = nullptr;

MediaPlayerFactory::MediaPlayerFactory()
{
    this->_log = Log::getInstance();
}

MediaPlayer *MediaPlayerFactory::createPlayer()
{
    auto *factory = getInstance();
    auto *vlcInstance = factory->getVlcInstance();

    if (vlcInstance == nullptr) {
        return nullptr;
    }

    auto *player = new MediaPlayer(vlcInstance);

    factory->_log->debug("Media player created");
    return player;
}

Media *MediaPlayerFactory::createMedia(const char *mrl)
{
    auto *factory = getInstance();
    auto *vlcInstance = factory->getVlcInstance();

    if (vlcInstance == nullptr) {
        return nullptr;
    }

    return new Media(mrl, vlcInstance);
}

libvlc_instance_t *MediaPlayerFactory::getVlcInstance()
{
    if (_vlcInstance == nullptr) {
        // initialize VLC args
        _log->trace("Creating new media player");
        const char *vlcArgs = _log->getLevel() & TRACE_FLAG ? "--verbose=2" : nullptr;
        int argc = _log->getLevel() & TRACE_FLAG ? 1 : 0;
        const char *const *argv = &vlcArgs;

        // create a new vlc instance
        _log->trace("Initializing new VLC instance");
        this->_vlcInstance = libvlc_new(argc, argv);

        // check if a vlc instance was created with success
        // if not, show an error dialog
        if (this->_vlcInstance == nullptr) {
            _log->error("Failed to initialize new VLC instance");
        }
    }

    return this->_vlcInstance;
}

MediaPlayerFactory *MediaPlayerFactory::getInstance()
{
    if (!instance) {
        instance = new MediaPlayerFactory();
    }

    return instance;
}
