#ifndef POPCORN_PLAYER_LIB_H
#define POPCORN_PLAYER_LIB_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct popcorn_player_t popcorn_player_t;

/**
 * Create a new Popcorn PopcornPlayer instance.
 *
 * @return Returns the Popcorn PopcornPlayer instance.
 */
popcorn_player_t *popcorn_player_new();

/**
 * Execute the Popcorn PopcornPlayer.
 * This function will exit when the Popcorn PopcornPlayer is exited with a status code.
 * It's recommended to run this on a separate thread which isn't disposed until the Popcorn PopcornPlayer exists.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 * @return Returns the exit code of the Popcorn PopcornPlayer.
 */
int popcorn_player_exec(popcorn_player_t *pdp);

/**
 * Release the Popcorn PopcornPlayer and release it's resources.
 *
 * @param pdp The Popcorn PopcornPlayer instance to release.
 */
void popcorn_player_release(popcorn_player_t *pdp);

/**
 * Play the given MRL in the Popcorn PopcornPlayer.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 * @param mrl The MRL to start playing.
 */
void popcorn_player_play(popcorn_player_t *pdp, const char *mrl);

/**
 * Pause the current playback of the Popcorn PopcornPlayer.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 */
void popcorn_player_pause(popcorn_player_t *pdp);

/**
 * Resume the playback of the Popcorn PopcornPlayer.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 */
void popcorn_player_resume(popcorn_player_t *pdp);

/**
 * Stop the current playback of the Popcorn PopcornPlayer.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 */
void popcorn_player_stop(popcorn_player_t *pdp);

/**
 * Show the Popcorn PopcornPlayer.
 *
 * @param pdp The Popcorn PopcornPlayer instance.
 */
void popcorn_player_show(popcorn_player_t *pdp);

#ifdef __cplusplus
}
#endif

#endif //POPCORN_PLAYER_LIB_H
