#ifndef POPCORNTIME_POPCORNKEYSLIB_H
#define POPCORNTIME_POPCORNKEYSLIB_H

#include "PopcornKeysCallbacks.h"
#ifdef __cplusplus
extern "C" {
#endif

typedef struct popcorn_keys_t popcorn_keys_t;

/**
 * Create a new Popcorn Keys instance.
 *
 * @return Returns the Popcorn Keys instance.
 */
popcorn_keys_t *popcorn_keys_new(int argc, char **argv);

/**
 * Release the Popcorn Keys instance.
 *
 * @param pk The Popcorn Keys instance.
 */
void popcorn_keys_release(popcorn_keys_t *pk);

/**
 * Register a new callback for the media keys.
 *
 * @param pk The Popcorn Keys instance.
 * @param callback The callback function.
 */
void popcorn_keys_media_callback(popcorn_keys_t *pk, popcorn_keys_media_key_pressed_t callback);

#ifdef __cplusplus
}
#endif

#endif //POPCORNTIME_POPCORNKEYSLIB_H
