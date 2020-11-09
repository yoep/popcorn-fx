#include <iostream>
#include "PlayerLib.h"
#include "PlayerWindow.h"

using namespace std;

struct popcorn_player_t {
    void *window;
};

popcorn_player_t *popcorn_player_new() {
    popcorn_player_t *pdp;

    // initialize the return type
    pdp = (typeof(pdp)) malloc(sizeof(*pdp));

    int argc = 0;

    // create a new PlayerWindow instance
    auto *window = new PlayerWindow(argc, nullptr);

    // assign the window to the return struct for later use
    pdp->window = window;

    return pdp;
}

int popcorn_player_exec(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return -1;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    return window->exec();
}

void popcorn_player_release(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->close();

    delete static_cast<PlayerWindow *>(pdp->window);
    free(pdp);
}

void popcorn_player_play(popcorn_player_t *pdp, const char *mrl) {
    if (pdp == nullptr)
        return;

    cout << std::string("Received popcorn_player_play mrl: ") + mrl << endl;
    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->play(mrl);
}

void popcorn_player_pause(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->getPlayer()->pause();
}

void popcorn_player_resume(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->getPlayer()->resume();
}

void popcorn_player_stop(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->stop();
}

void popcorn_player_show(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->show();
}

void popcorn_player_show_maximized(popcorn_player_t *pdp) {
    if (pdp == nullptr)
        return;

    cout << std::string("Received popcorn_player_show_maximized") << endl;
    PlayerWindow *window;

    window = static_cast<PlayerWindow *>(pdp->window);
    window->showMaximized();
}