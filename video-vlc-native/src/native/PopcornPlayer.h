#ifndef POPCORN_PLAYER_PLAYERWINDOW_H
#define POPCORN_PLAYER_PLAYERWINDOW_H

#include "Log.h"
#include "PopcornPlayerCallbacks.h"
#include "PopcornPlayerEventManager.h"
#include "widgets/PopcornPlayerWindow.h"
#include "widgets/VideoWidget.h"

#include <QGuiApplication>
#include <QtWidgets/QMainWindow>
#include <player/MediaPlayer.h>

class PopcornPlayer {
public:
    /**
     * Initialize a new popcorn player instance.
     *
     * @param argc The number of arguments for the new instance.
     * @param argv The arguments for the new instance.
     */
    PopcornPlayer(int &argc, char **argv);

    ~PopcornPlayer();

    /**
     * Show the popcorn player.
     * If the player is already shown, then this action has no effect.
     */
    void show();

    /**
     * Close the popcorn player.
     * This will quit the player instance.
     */
    void close();

    /**
     * Play the given MRL in the player.
     *
     * @param mrl The MRL to play.
     */
    void play(const char *mrl);

    /**
     * Seek the given time within the current media playback.
     * This has no effect if no media is currently being played.
     *
     * @param time The time to seek in millis.
     */
    void seek(long time);

    /**
     * Pause the current media playback.
     * This has no effect if no media is currently being played.
     */
    void pause();

    /**
     * Resume the current media playback.
     * This has no effect if no media is currently being played.
     */
    void resume();

    /**
     * Stop the current media playback.
     * This will automatically hide the player.
     *
     * If no media playback if currently playing, the window will be still hidden.
     */
    void stop();

    /**
     * Set the subtitle file for the current media playback.
     * This has no effect if no media is currently being played.
     *
     * @param uri The file uri to the subtitle file.
     */
    void setSubtitleFile(const char *uri);

    /**
     * Set the subtitle delay for the current media playback.
     *
     * @param delay The delay of the subtitle.
     */
    void setSubtitleDelay(long delay);

    /**
     * Set the fullscreen mode of the popcorn player.
     * If the player is currently hidden, it will be shown maximized or in fullscreen if this method is triggered.
     *
     * @param fullscreen The indication if the player needs to be shown in fullscreen.
     */
    void setFullscreen(bool fullscreen);

    /**
     * Register a new state callback for this player.
     * If the player state is changed, the given callback will be triggered with the new state.
     *
     * @param callback The callback which needs to be triggered.
     */
    void registerStateCallback(popcorn_player_state_callback_t callback);

    /**
     * Register a new time callback for this player.
     * If the player time is changed, the given callback will be triggered with the new time.
     *
     * @param callback The callback which needs to be triggered.
     */
    void registerTimeCallback(popcorn_player_time_callback_t callback);

    /**
     * Register a new duration callback for this player.
     * If the player duration is changed, the given callback will be triggered with the new duration.
     *
     * @param callback The callback which needs to be triggered.
     */
    void registerDurationCallback(popcorn_player_duration_callback_t callback);

private:
    int &_argc;
    char **_argv;
    std::shared_ptr<PopcornPlayerWindow> _window;
    std::shared_ptr<MediaPlayer> _mediaPlayer;
    std::shared_ptr<PopcornPlayerEventManager> _eventManager;
    int _fontAwesomeRegularId = -1;
    int _fontAwesomeSolidId = -1;
    int _openSansBoldId = -1;
    int _openSansRegularId = -1;
    int _openSansSemiBoldId = -1;
    Log *_log;

    void init();

    void loadFonts();

    void parseArguments();

    void updateLogLevel(char *levelArg);

    bool waitForEventManager();

    static void loadIcon();
};

#endif // POPCORN_PLAYER_PLAYERWINDOW_H
