#include "PopcornPlayerEventManager.h"

PopcornPlayerEventManager::PopcornPlayerEventManager(MediaPlayer *mediaPlayer)
{
    this->_log = Log::instance();
    this->_stateCallbacks = {};
    this->_timeCallbacks = {};
    this->_durationCallbacks = {};

    connectEvents(mediaPlayer);
}

void PopcornPlayerEventManager::addStateCallback(popcorn_player_state_callback_t callback)
{
    if (callback == nullptr) {
        _log->error("Unable to add NULL as player state callback");
        return;
    }

    _log->trace("Adding new state callback to the event manager");
    _stateCallbacks.push_back(callback);
    _log->debug("State callback has been registered in the event manager");
}

void PopcornPlayerEventManager::addTimeCallback(popcorn_player_time_callback_t callback)
{
    if (callback == nullptr) {
        _log->error("Unable to add NULL as player time callback");
        return;
    }

    _log->trace("Adding new time callback to the event manager");
    _timeCallbacks.push_back(callback);
    _log->debug("Time callback has been registered in the event manager");
}

void PopcornPlayerEventManager::addDurationCallback(popcorn_player_duration_callback_t callback)
{
    if (callback == nullptr) {
        _log->error("Unable to add NULL as player duration callback");
        return;
    }

    _log->trace("Adding new duration callback to the event manager");
    _durationCallbacks.push_back(callback);
    _log->debug("Duration callback has been registered in the event manager");
}

void PopcornPlayerEventManager::onStateChanged(MediaPlayerState newState)
{
    _log->trace(std::string("Event manager received new player state ") + media_player_state_as_string(newState));
    for (auto const &callback : _stateCallbacks) {
        callback(newState);
    }
}

void PopcornPlayerEventManager::onTimeChanged(long newValue)
{
    _log->trace(std::string("Event manager received new player time ") + std::to_string(newValue));
    for (auto const &callback : _timeCallbacks) {
        callback(newValue);
    }
}

void PopcornPlayerEventManager::onDurationChanged(long newValue)
{
    _log->trace(std::string("Event manager received new player duration ") + std::to_string(newValue));
    for (auto const &callback : _durationCallbacks) {
        callback(newValue);
    }
}

void PopcornPlayerEventManager::connectEvents(MediaPlayer *mediaPlayer)
{
    _log->trace("Initializing event manager");
    connect(mediaPlayer, &MediaPlayer::stateChanged,
        this, &PopcornPlayerEventManager::onStateChanged);
    connect(mediaPlayer, &MediaPlayer::timeChanged,
        this, &PopcornPlayerEventManager::onTimeChanged);
    connect(mediaPlayer, &MediaPlayer::durationChanged,
        this, &PopcornPlayerEventManager::onDurationChanged);
    _log->debug("Event manager has been initialized");
}
