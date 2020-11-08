#include <thread>
#include <QtCore/QCoreApplication>
#include <getopt.h>
#include <iostream>
#include "PlayerWindow.h"

using namespace std;

int main(int argc, char *argv[]) {
    auto *window = new PlayerWindow(argc, argv);

    // an example of the JNA invoking the initial window on a separate thread
    std::thread t1([&] {
        window->exec();
    });
    t1.detach();

    // wait some time for the application to initialize
    std::this_thread::sleep_for(std::chrono::milliseconds(5000));

    // run some QT options on another thread that the current QApplication thread
    window->showMaximized();
    window->play(argv[optind]);

    // keep the main thread alive for some additional time
    std::this_thread::sleep_for(std::chrono::milliseconds(60000));
    return 0;
}
