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

    int exec();

    void show();

    void close();

    void play(const char *mrl);

    void pause();

    void resume();

    void stop();

    void setSubtitleFile(const char* uri);

    void setSubtitleDelay(long delay);

    void setFullscreen(bool fullscreen);

private:
    int &argc;
    char **argv;
    QApplication *app;
    PopcornPlayerWindow *window;
    MediaPlayer *mediaPlayer;
    Log *log;

    template <typename Func>
    void invokeOnQt(Func func);

    void loadFonts();

    void parseArguments();

    void updateLogLevel(char *levelArg);

    static void loadIcon();
};

#endif // POPCORN_PLAYER_PLAYERWINDOW_H
