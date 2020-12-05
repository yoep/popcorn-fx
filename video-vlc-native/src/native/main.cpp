#include "PopcornPlayerLib.h"

#include <getopt.h>
#include <iostream>
#include <thread>

using namespace std;

// THIS IS A TEST CASE SCENARIO
// ONLY USE THE LIBRARY FOR PLAYBACKS AND NOT THE RUNNER
int main(int argc, char *argv[])
{
    popcorn_player_t *instance = popcorn_player_new(argc, argv);

    std::this_thread::sleep_for(std::chrono::milliseconds(1000));

    // run some QT options on another thread that the current QApplication thread
    std::thread playThread([&, instance] {
        popcorn_player_show(instance);
        popcorn_player_play(instance, argv[optind]);
    });
    playThread.join();

    std::this_thread::sleep_for(std::chrono::milliseconds(20000));
    std::thread t3([&, instance] {
        popcorn_player_pause(instance);
    });
    t3.detach();

    std::this_thread::sleep_for(std::chrono::milliseconds(2000));
    popcorn_player_resume(instance);

    // keep the main thread alive for some additional time
    std::this_thread::sleep_for(std::chrono::milliseconds(10000));
    popcorn_player_release(instance);

    return 0;
}
