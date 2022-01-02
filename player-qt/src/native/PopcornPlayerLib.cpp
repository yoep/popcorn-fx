#include "PopcornPlayerLib.h"

#include "PopcornPlayer.h"

#include <iostream>

using namespace std;

struct popcorn_player_t {
    void *player;
};

popcorn_player_t *popcorn_player_new(int argc, char **argv)
{
    popcorn_player_t *pdp;

    // initialize the return type
    pdp = (typeof(pdp))malloc(sizeof(*pdp));

    // create a new PopcornPlayer instance
    auto *player = new PopcornPlayer(argc, argv);

    // assign the player to the return struct for later use
    pdp->player = player;

    return pdp;
}

void popcorn_player_release(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);

    try {
        player->close();

        delete static_cast<PopcornPlayer *>(pdp->player);
        free(pdp);
    } catch (std::exception &ex) {
        cerr << "Failed to release Popcorn Player instance, " << ex.what() << endl;
    }
}

void popcorn_player_play(popcorn_player_t *pdp, const char *mrl)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->play(mrl);
}

void popcorn_player_seek(popcorn_player_t *pdp, const char *time)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    auto seekTime = std::stol(time);
    player->seek(seekTime);
}

void popcorn_player_pause(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->pause();
}

void popcorn_player_resume(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->resume();
}

void popcorn_player_stop(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->stop();
}

void popcorn_player_show(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->show();
}

void popcorn_player_fullscreen(popcorn_player_t *pdp, bool fullscreen)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->setFullscreen(fullscreen);
}

void popcorn_player_subtitle(popcorn_player_t *pdp, const char *uri)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->setSubtitleFile(uri);
}

void popcorn_player_subtitle_delay(popcorn_player_t *pdp, long delay)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->setSubtitleDelay(delay);
}

int popcorn_player_volume(popcorn_player_t *pdp)
{
    if (pdp == nullptr)
        return -1;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    return 0;
}

void popcorn_player_set_volume(popcorn_player_t *pdp, int volume)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
}

void popcorn_player_state_callback(popcorn_player_t *pdp, popcorn_player_state_callback_t callback)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->registerStateCallback(callback);
}

void popcorn_player_time_callback(popcorn_player_t *pdp, popcorn_player_time_callback_t callback)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->registerTimeCallback(callback);
}

void popcorn_player_duration_callback(popcorn_player_t *pdp, popcorn_player_duration_callback_t callback)
{
    if (pdp == nullptr)
        return;

    PopcornPlayer *player;

    player = static_cast<PopcornPlayer *>(pdp->player);
    player->registerDurationCallback(callback);
}
