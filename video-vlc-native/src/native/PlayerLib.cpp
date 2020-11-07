#include "PlayerLib.h"
#include "PlayerWindow.h"

popcorn_desktop_player *popcorn_desktop_new(int argc, char **argv) {
    auto *window = new PlayerWindow(argc, argv);

    return nullptr;
}