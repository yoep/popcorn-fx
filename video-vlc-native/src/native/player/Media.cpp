#include "Media.h"

#include <regex>

using namespace std;

Media::Media(const char *mrl, libvlc_instance_t *vlcInstance)
{
    this->_log = Log::instance();
    this->_vlcInstance = vlcInstance;
    this->_vlcMedia = nullptr;
    this->_vlcEvent = nullptr;
    this->_mrl = mrl;

    initializeMedia();
}

Media::~Media()
{
    unsubscribeEvents();
    libvlc_media_release(_vlcMedia);
}

libvlc_media_t *Media::vlcMedia()
{
    return this->_vlcMedia;
}

long Media::getDuration()
{
    return libvlc_media_get_duration(_vlcMedia);
}

void Media::initializeMedia()
{
    _log->trace(std::string("Initializing media for ") + _mrl);

    // create the VLC media item based on the MRL type
    if (isHttpUrl(this->_mrl)) {
        this->_vlcMedia = createFromUrl(this->_mrl);
    } else {
        this->_vlcMedia = createFromFile(this->_mrl);
    }

    // subscribe to the media VLC events if the media was created with success
    if (this->_vlcMedia != nullptr) {
        _log->trace("Creating new VLC event manager for the media");
        _vlcEvent = libvlc_media_event_manager(_vlcMedia);
        subscribeEvents();
    }
}

libvlc_media_t *Media::createFromFile(const char *path)
{
    _log->debug(std::string("Creating media for file path: ") + path);
    auto *media = libvlc_media_new_path(_vlcInstance, path);

    if (media == nullptr) {
        _log->warn(std::string("Unable to create media for path ") + path);
        return nullptr;
    }

    _log->debug(std::string("Media has been created with success for ") + path);
    return media;
}

libvlc_media_t *Media::createFromUrl(const char *url)
{
    _log->debug(std::string("Creating media for url: ") + url);
    auto *media = libvlc_media_new_location(_vlcInstance, url);

    if (media == nullptr) {
        _log->warn(std::string("Unable to create media for url ") + url);
        return nullptr;
    }

    _log->debug(std::string("Media has been created with success for ") + url);
    return media;
}

void Media::subscribeEvents()
{
    if (_vlcEvent == nullptr) {
        _log->warn("Unable to subscribe to VLC events, no VLC event manager present");
        return;
    }

    _log->trace("Subscribing to VLC media events");
    foreach (const libvlc_event_e event, eventList()) {
        libvlc_event_attach(_vlcEvent, event, vlcCallback, this);
    }
    _log->debug("Subscribed to VLC media events");
}

void Media::unsubscribeEvents()
{
    if (_vlcEvent == nullptr) {
        _log->warn("Unable to unsubscribe from VLC events, no VLC event manager present");
        return;
    }

    _log->trace("Unsubscribing from VLC media events");
    foreach (const libvlc_event_e event, eventList()) {
        libvlc_event_detach(_vlcEvent, event, vlcCallback, this);
    }
    _log->debug("Unsubscribed from VLC media events");
}

void Media::vlcCallback(const libvlc_event_t *event, void *instance)
{
    Log *log = Log::instance();

    // check if the instance is valid
    // if not, throw an error as we'll be unable to do anything with the event
    if (instance == nullptr) {
        log->error("Invalid VLC callback event, instance is NULL");
    }

    auto *media = static_cast<Media *>(instance);

    switch (event->type) {
    case libvlc_MediaDurationChanged:
        emit media->durationChanged(event->u.media_duration_changed.new_duration);
        break;
    case libvlc_MediaStateChanged:

        break;
    case libvlc_MediaFreed:

        break;
    case libvlc_MediaParsedChanged:

        break;
    default:
        log->warn(std::string("Unknown VLC media event type ") + std::to_string(event->type));
        break;
    }
}

bool Media::isHttpUrl(const char *mrl)
{
    std::string value = mrl;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}

QList<libvlc_event_e> Media::eventList()
{
    QList<libvlc_event_e> eventList;
    eventList << libvlc_MediaDurationChanged;
    eventList << libvlc_MediaStateChanged;
    eventList << libvlc_MediaFreed;
    eventList << libvlc_MediaParsedChanged;

    return eventList;
}
