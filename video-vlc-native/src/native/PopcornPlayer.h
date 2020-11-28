#ifndef POPCORN_PLAYER_PLAYERWINDOW_H
#define POPCORN_PLAYER_PLAYERWINDOW_H

#include "Log.h"
#include "widgets/PopcornPlayerWindow.h"
#include "widgets/VideoWidget.h"

#include <QGuiApplication>
#include <QtWidgets/QMainWindow>
#include <player/MediaPlayer.h>

class PopcornPlayer {
public:
    PopcornPlayer(int &argc, char **argv);

    ~PopcornPlayer();

    void show();

    void close();

    void play(const char *mrl);

    void pause();

    void resume();

    void stop();

    void setSubtitleFile(const char *uri);

    void setSubtitleDelay(long delay);

    void setFullscreen(bool fullscreen);

private:
    int &_argc;
    char **_argv;
    std::shared_ptr<PopcornPlayerWindow> _window;
    std::shared_ptr<MediaPlayer> _mediaPlayer;
    Log *_log;

    void init();

    void loadFonts();

    void parseArguments();

    void updateLogLevel(char *levelArg);

    static void loadIcon();
};

#endif // POPCORN_PLAYER_PLAYERWINDOW_H
