#include <thread>
#include <getopt.h>
#include "PlayerLib.h"

using namespace std;

int main(int argc, char *argv[]) {
    popcorn_player_t *instance = popcorn_player_new();

    // an example of the JNA invoking the initial window on a separate thread
    std::thread t1([&, instance] {
        popcorn_player_exec(instance);
    });
    t1.detach();

    // wait some time for the application to initialize
    std::this_thread::sleep_for(std::chrono::milliseconds(5000));

    // run some QT options on another thread that the current QApplication thread
    std::thread t2([&, instance] {
        popcorn_player_show_maximized(instance);
        popcorn_player_play(instance, argv[optind]);
    });
    t2.detach();

    std::this_thread::sleep_for(std::chrono::milliseconds(10000));
    std::thread t3([&, instance] {
        popcorn_player_pause(instance);
    });
    t3.detach();

    std::this_thread::sleep_for(std::chrono::milliseconds(2000));
    popcorn_player_resume(instance);

    // keep the main thread alive for some additional time
    std::this_thread::sleep_for(std::chrono::milliseconds(30000));
    return 0;
}
