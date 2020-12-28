#ifndef POPCORNTIME_POPCORNKEYSCALLBACKS_H
#define POPCORNTIME_POPCORNKEYSCALLBACKS_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Callback function for when a media key has been pressed.
 *
 * @param keyType The media key type which has been pressed.
 */
typedef void (*popcorn_keys_media_key_pressed_t)(int keyType);

#ifdef __cplusplus
}
#endif

#endif //POPCORNTIME_POPCORNKEYSCALLBACKS_H
