
#include "PopcornKeysLib.h"

#include <chrono>
#include <thread>

int main(int argc, char *argv[])
{
    popcorn_keys_t *instance = popcorn_keys_new(argc, argv);

    std::this_thread::sleep_for(std::chrono::milliseconds(20000));

    popcorn_keys_release(instance);

    return 0;
}