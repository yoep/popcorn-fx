#include <thread>
#include "PlayerLib.h"
#include "PlayerWindow.h"

struct popcorn_desktop_player_t {
    void *window;
};

popcorn_desktop_player_t *popcorn_desktop_player_new() {
    popcorn_desktop_player_t *pdp;

    // initialize the return type
    pdp = (typeof(pdp)) malloc(sizeof(*pdp));

    int argc = 0;
    char **argv = nullptr;

    // create a new PlayerWindow instance
    auto *window = new PlayerWindow(argc, argv);

    // assign the window to the return struct for later use
    pdp->window = window;

    return pdp;
}

int popcorn_desktop_player_exec(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return -1;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    return window->exec();
}

void popcorn_desktop_player_release(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->close();

    delete static_cast<PlayerWindow *>(pdp->window);
    free(pdp);
}

void popcorn_desktop_player_play(popcorn_desktop_player_t *pdp, const char *mrl) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->play(mrl);
}

void popcorn_desktop_player_pause(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->getPlayer()->pause();
}

void popcorn_desktop_player_resume(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->getPlayer()->resume();
}

void popcorn_desktop_player_stop(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->getPlayer()->stop();
}

void popcorn_desktop_player_show(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->show();
}

void popcorn_desktop_player_show_maximized(popcorn_desktop_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->showMaximized();
}