#include <QtCore/QCoreApplication>
#include <QtWidgets/QApplication>
#include <iostream>
#include <regex>
#include <QtWidgets/QStackedLayout>
#include "PlayerWindow.h"
#include "VideoPlayer.h"

using namespace std;

const char *const ApplicationTitle = "Popcorn Player";

PlayerWindow::PlayerWindow(int &argc, char **argv) : argc(argc) {
    this->argv = argv;
    this->app = nullptr;
    this->window = nullptr;
    this->player = nullptr;
}

VideoPlayer *PlayerWindow::getPlayer() const {
    return player;
}

int PlayerWindow::exec() {
    cout << "Initializing Qt Application" << endl;
    QCoreApplication::setApplicationName(ApplicationTitle);
    this->app = new QApplication(argc, argv);

    // create a new video player
    player = new VideoPlayer();

    window = new QWidget();
    auto *layout = new QStackedLayout();
    layout->addWidget(player);
    window->setLayout(layout);

    // make the  QT window undecorated
    window->setWindowFlags(Qt::FramelessWindowHint);
    window->setWindowFlag(Qt::Window);

    int exit = QApplication::exec();
    cout << "QApplication finished " << endl;
    return exit;
}

bool PlayerWindow::isInitialized() {
    // check if the app & player have been assigned
    return this->app != nullptr && this->player != nullptr;
}

void PlayerWindow::show() {
    if (window == nullptr) {
        cerr << "QT Window is not initialized" << endl;
        return;
    }

    QMetaObject::invokeMethod(this->app, [&] {
        cout << "Showing QT Player Window" << endl;
        window->showNormal();
    });
}

void PlayerWindow::showMaximized() {
    if (window == nullptr) {
        cerr << "QT Window is not initialized" << endl;
        return;
    }

    QMetaObject::invokeMethod(this->app, [&] {
        cout << "Showing QT Player Window" << endl;
        window->showMaximized();
    });
}

void PlayerWindow::close() {
    if (this->app == nullptr) {
        cerr << "QT Application has not been initialized" << endl;
        return;
    }

    QMetaObject::invokeMethod(this->app, [&] {
        player->close();
        QApplication::exit(0);
    });
}

void PlayerWindow::play(const char *mrl) {
    if (mrl == nullptr) {
        cerr << "No MRL has been passed to the play function, ignoring play action" << endl;
        return;
    }

    if (this->app == nullptr) {
        cerr << "QT Application has not been initialized" << endl;
        return;
    }

    QMetaObject::invokeMethod(this->app, [&, mrl] {
        if (isHttpUrl(mrl)) {
            player->playUrl(mrl);
        } else {
            player->playFile(mrl);
        }
    });
}

bool PlayerWindow::isMaximized() {
    return player->isMaximized();
}

void PlayerWindow::setMaximized(bool maximized) {
    if (maximized) {
        player->showMaximized();
    } else {
        player->showNormal();
    }
}

bool PlayerWindow::isHttpUrl(const char *url) {
    std::string value = url;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}
