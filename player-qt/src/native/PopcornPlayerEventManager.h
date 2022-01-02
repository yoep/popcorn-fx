#ifndef POPCORNPLAYER_POPCORNPLAYEREVENTMANAGER_H
#define POPCORNPLAYER_POPCORNPLAYEREVENTMANAGER_H

#include "PopcornPlayerCallbacks.h"

#include <QObject>
#include <QtCore/QMetaObject>
#include <player/MediaPlayer.h>
#include <player/MediaPlayerState.h>

using namespace std;

class PopcornPlayerEventManager : public QObject {
    Q_OBJECT

public:
    /**
     * Initialize a new event manager to the popcorn player.
     *
     * @param mediaPlayer The media player to which this event manager listens.
     */
    PopcornPlayerEventManager(MediaPlayer *mediaPlayer);

    void addStateCallback(popcorn_player_state_callback_t callback);

    void addTimeCallback(popcorn_player_time_callback_t callback);

    void addDurationCallback(popcorn_player_duration_callback_t callback);

public slots:
    void onStateChanged(MediaPlayerState newState);

    void onTimeChanged(long newValue);

    void onDurationChanged(long newValue);

private:
    std::list<popcorn_player_state_callback_t> _stateCallbacks;
    std::list<popcorn_player_time_callback_t> _timeCallbacks;
    std::list<popcorn_player_duration_callback_t> _durationCallbacks;
    Log *_log;

    void connectEvents(MediaPlayer *mediaPlayer);
};

#endif //POPCORNPLAYER_POPCORNPLAYEREVENTMANAGER_H
