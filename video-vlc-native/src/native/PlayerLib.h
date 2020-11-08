#ifndef POPCORNDESKTOPPLAYER_PLAYERLIB_H
#define POPCORNDESKTOPPLAYER_PLAYERLIB_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct popcorn_player_t popcorn_player_t;

popcorn_player_t *popcorn_player_new();

int popcorn_player_exec(popcorn_player_t *pdp);

void popcorn_player_release(popcorn_player_t *pdp);

void popcorn_player_play(popcorn_player_t *pdp, const char *mrl);

void popcorn_player_pause(popcorn_player_t *pdp);

void popcorn_player_resume(popcorn_player_t *pdp);

void popcorn_player_stop(popcorn_player_t *pdp);

void popcorn_player_show(popcorn_player_t *pdp);

void popcorn_player_show_maximized(popcorn_player_t *pdp);

#ifdef __cplusplus
}
#endif

#endif //POPCORNDESKTOPPLAYER_PLAYERLIB_H
