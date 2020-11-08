#ifndef POPCORNDESKTOPPLAYER_PLAYERLIB_H
#define POPCORNDESKTOPPLAYER_PLAYERLIB_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct popcorn_desktop_player_t popcorn_desktop_player_t;

popcorn_desktop_player_t *popcorn_desktop_player_new();

int popcorn_desktop_player_exec(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_release(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_play(popcorn_desktop_player_t *pdp, const char *mrl);

void popcorn_desktop_player_pause(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_resume(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_stop(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_show(popcorn_desktop_player_t *pdp);

void popcorn_desktop_player_show_maximized(popcorn_desktop_player_t *pdp);

#ifdef __cplusplus
}
#endif

#endif //POPCORNDESKTOPPLAYER_PLAYERLIB_H
