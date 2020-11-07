#include <QtCore/QCoreApplication>
#include <QtWidgets/QApplication>
#include <iostream>
#include <getopt.h>
#include <regex>
#include "PlayerWindow.h"
#include "VideoPlayer.h"

using namespace std;

static const char *const ApplicationTitle = "Popcorn Time VideoPlayer";

PlayerWindow::PlayerWindow(int &argc, char **argv) : argc(argc) {
    this->argv = argv;
    this->player = nullptr;
}

int PlayerWindow::exec() {
    cout << "Creating QT application" << endl;
    QCoreApplication::setApplicationName(ApplicationTitle);
    QApplication app(argc, argv);

    // create a new video player
    player = new VideoPlayer();

    // make the  QT window undecorated
    cout << "Configuring QT Window Flags" << endl;
    player->setWindowFlag(Qt::FramelessWindowHint);

    // maximize the QT window on startup
    cout << "Showing QT window" << endl;
    player->showMaximized();

    // retrieve the playback url from the command line
    char *url = parseArguments();

    // check if a playback url was retrieved
    if (url != nullptr) {
        play(url);
    }

    return QApplication::exec();
}

void PlayerWindow::play(char *mrl) {
    if (mrl == nullptr)
        return;

    if (isHttpUrl(mrl)) {
        player->playUrl(mrl);
    } else {
        player->playFile(mrl);
    }
}

char *PlayerWindow::parseArguments() {
    return argv[optind];
}

bool PlayerWindow::isHttpUrl(char *url) {
    std::string value = url;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}
