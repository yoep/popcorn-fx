#include "PlayerWindow.h"

int main(int argc, char *argv[]) {
    auto *window = new PlayerWindow(argc, argv);
    return window->exec();
}
