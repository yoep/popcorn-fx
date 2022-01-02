#ifndef POPCORNPLAYER_POPCORNPLAYERCALLBACKS_H
#define POPCORNPLAYER_POPCORNPLAYERCALLBACKS_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Callback function for the popcorn player state change.
 *
 * @param newState The new popcorn player state.
 */
typedef void (*popcorn_player_state_callback_t)(int newState);

/**
 * Callback function for the popcorn player time change.
 *
 * @param newTime The new player time in millis.
 */
typedef void (*popcorn_player_time_callback_t)(const char *newTime);
/**
 * Callback function for the popcorn player duration change.
 *
 * @param newDuration The new player duration in millis.
 */
typedef void (*popcorn_player_duration_callback_t)(const char *newDuration);

#ifdef __cplusplus
}
#endif

#endif //POPCORNPLAYER_POPCORNPLAYERCALLBACKS_H
