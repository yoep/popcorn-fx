#include "Media.h"

#include <QApplicationManager.h>
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

libvlc_media_list_t *Media::subitems()
{
    return libvlc_media_subitems(_vlcMedia);
}

bool Media::hasSubitems()
{
    return countSubitems() > 0;
}

MediaState Media::state()
{
    return this->_state;
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
    } else {
        invokeStateChange(MediaState::ERROR);
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

    invokeStateChange(MediaState::PARSING);
    auto parseResult = libvlc_media_parse_with_options(media, libvlc_media_parse_flag_t::libvlc_media_parse_network, 30000);

    if (parseResult == -1) {
        _log->warn(std::string("Failed to start parsing of media url ") + url);
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

void Media::updateState(int vlcState)
{
    _log->trace("Parsing new VLC media item state");
    MediaState newState;

    switch (vlcState) {
    case 1:
        newState = MediaState::OPENING;
        break;
    case 3:
        newState = MediaState::PLAYING;
        break;
    case 4:
        newState = MediaState::PAUSED;
        break;
    case 6:
        newState = MediaState::ENDED;
        break;
    case 7:
        newState = MediaState::ERROR;
        break;
    default:
        _log->warn("Unknown VLC media item state " + std::to_string(vlcState));
        newState = MediaState::UNKNOWN;
        break;
    }

    invokeStateChange(newState);
}

void Media::onParsedEvent()
{
    _log->debug("Found a total of " + std::to_string(countSubitems()) + " media sub items");
    invokeStateChange(MediaState::PARSED);
    emit this->parsed();
}

int Media::countSubitems()
{
    return libvlc_media_list_count(subitems());
}

void Media::invokeStateChange(MediaState newState)
{
    // check if the state is the same as the current known state
    // if so, ignore the state update
    if (newState == this->_state) {
        return;
    }

    // store the new state
    this->_state = newState;

    _log->debug(string("Media item state changed to ") + media_state_as_string(_state));
    emit stateChanged(_state);
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
    case libvlc_MediaStateChanged:
        media->updateState(event->u.media_state_changed.new_state);
        break;
    case libvlc_MediaParsedChanged:
        media->onParsedEvent();
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
    eventList << libvlc_MediaStateChanged;
    eventList << libvlc_MediaParsedChanged;

    return eventList;
}
