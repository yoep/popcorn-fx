#ifndef POPCORN_PLAYER_LIB_H
#define POPCORN_PLAYER_LIB_H

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
 * Execute the Popcorn Player.
 * This function will exit when the Popcorn Player is exited with a status code.
 * It's recommended to run this on a separate thread which isn't disposed until the Popcorn Player exists.
 *
 * @param pdp The Popcorn Player instance.
 * @return Returns the exit code of the Popcorn Player.
 */
int popcorn_player_exec(popcorn_player_t *pdp);

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

#ifdef __cplusplus
}
#endif

#endif //POPCORN_PLAYER_LIB_H
