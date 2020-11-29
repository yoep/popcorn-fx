#ifndef POPCORN_PLAYER_LIB_H
#define POPCORN_PLAYER_LIB_H

#include "PopcornPlayerCallbacks.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct popcorn_player_t popcorn_player_t;

/**
 * Create a new Popcorn Player instance.
 *
 * @return Returns the Popcorn Player instance.
 */
popcorn_player_t *popcorn_player_new(int argc, char **argv);

/**
 * Release the Popcorn Player and release it's resources.
 *
 * @param pdp The Popcorn Player instance to release.
 */
void popcorn_player_release(popcorn_player_t *pdp);

/**
 * Play the given MRL in the Popcorn Player.
 *
 * @param pdp The Popcorn Player instance.
 * @param mrl The MRL to start playing.
 */
void popcorn_player_play(popcorn_player_t *pdp, const char *mrl);

/**
 * Seek the given time within the current playback of the Popcorn Player.
 *
 * @param pdp  The Popcorn Player instance.
 * @param time The time to seek in millis.
 */
void popcorn_player_seek(popcorn_player_t *pdp, long time);

/**
 * Pause the current playback of the Popcorn Player.
 *
 * @param pdp The Popcorn Player instance.
 */
void popcorn_player_pause(popcorn_player_t *pdp);

/**
 * Resume the playback of the Popcorn Player.
 *
 * @param pdp The Popcorn Player instance.
 */
void popcorn_player_resume(popcorn_player_t *pdp);

/**
 * Stop the current playback of the Popcorn Player.
 *
 * @param pdp The Popcorn Player instance.
 */
void popcorn_player_stop(popcorn_player_t *pdp);

/**
 * Show the Popcorn Player.
 *
 * @param pdp The Popcorn Player instance.
 */
void popcorn_player_show(popcorn_player_t *pdp);

/**
 * Change the fullscreen mode of the popcorn player.
 *
 * @param pdp The Popcorn Player instance.
 * @param fullscreen The fullscreen switch.
 */
void popcorn_player_fullscreen(popcorn_player_t *pdp, bool fullscreen);

/**
 * Add the given subtitle file uri to the current media playback.
 *
 * @param pdp The Popcorn Player instance.
 * @param uri The subtitle file uri to add.
 */
void popcorn_player_subtitle(popcorn_player_t *pdp, const char *uri);

/**
 * Update the subtitle delay for the current media playback.
 *
 * @param pdp The Popcorn Player instance.
 * @param delay The delay in micro seconds.
 */
void popcorn_player_subtitle_delay(popcorn_player_t *pdp, long delay);

/**
 * Register a callback for when the player state is being changed.
 *
 * @param pdp The Popcorn Player instance.
 * @param callback The callback function.
 */
void popcorn_player_state_callback(popcorn_player_t *pdp, popcorn_player_state_callback_t callback);

/**
 * Register a callback for when the player time is being changed.
 *
 * @param pdp The Popcorn Player instance.
 * @param callback The callback function.
 */
void popcorn_player_time_callback(popcorn_player_t *pdp, popcorn_player_time_callback_t callback);

/**
 * Register a callback for when the player duration is being changed.
 *
 * @param pdp The Popcorn Player instance.
 * @param callback The callback function.
 */
void popcorn_player_duration_callback(popcorn_player_t *pdp, popcorn_player_duration_callback_t callback);

#ifdef __cplusplus
}
#endif

#endif //POPCORN_PLAYER_LIB_H
