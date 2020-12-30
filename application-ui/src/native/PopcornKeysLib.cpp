#include "PopcornKeysLib.h"

#include "PopcornKeys.h"

#include <exception>
#include <malloc.h>

using namespace std;

struct popcorn_keys_t {
    void *keys;
};

popcorn_keys_t *popcorn_keys_new(int argc, char **argv)
{
    popcorn_keys_t *pk;

    // initialize the return type
    pk = (typeof(pk))malloc(sizeof(*pk));

    // create a new PopcornPlayer instance
    auto *keys = new PopcornKeys(argc, argv);

    // assign the player to the return struct for later use
    pk->keys = keys;

    return pk;
}

void popcorn_keys_release(popcorn_keys_t *pk)
{
    if (pk == nullptr)
        return;

    PopcornKeys *keys;
    keys = static_cast<PopcornKeys *>(pk->keys);

    delete keys;
    free(pk);
}

void popcorn_keys_media_callback(popcorn_keys_t *pk, popcorn_keys_media_key_pressed_t callback)
{
    if (pk == nullptr)
        return;

    PopcornKeys *keys;
    keys = static_cast<PopcornKeys *>(pk->keys);

    keys->addOnMediaKeyPressedCallback(callback);
}
