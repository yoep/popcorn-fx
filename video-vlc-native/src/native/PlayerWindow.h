#ifndef POPCORNDESKTOPPLAYER_PLAYERWINDOW_H
#define POPCORNDESKTOPPLAYER_PLAYERWINDOW_H


#include "VideoPlayer.h"

class PlayerWindow {
public:
    PlayerWindow(int &argc, char **argv);

    void play(char *mrl);

    int exec();

private:
    int &argc;
    char **argv;
    VideoPlayer *player;

    char *parseArguments();

    static bool isHttpUrl(char *string);
};


#endif //POPCORNDESKTOPPLAYER_PLAYERWINDOW_H
