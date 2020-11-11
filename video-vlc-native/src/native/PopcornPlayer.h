#ifndef POPCORN_PLAYER_PLAYERWINDOW_H
#define POPCORN_PLAYER_PLAYERWINDOW_H

#include "VideoPlayer.h"
#include "PopcornPlayerWindow.h"

#include <QGuiApplication>
#include <QtWidgets/QMainWindow>

class PopcornPlayer {
public:
    PopcornPlayer(int &argc, char **argv);

    int exec();

    void show();

    void showMaximized();

    void close();

    bool isInitialized();

    bool isMaximized();

    void setMaximized(bool maximized);

    void play(const char *mrl);

    void pause();

    void resume();

    void stop();

private:
    int &argc;
    char **argv;
    QApplication *app;
    PopcornPlayerWindow *window;

    static bool isHttpUrl(const char *string);

    template<typename Func>
    void invokeOnQt(Func func);
};

#endif // POPCORN_PLAYER_PLAYERWINDOW_H
