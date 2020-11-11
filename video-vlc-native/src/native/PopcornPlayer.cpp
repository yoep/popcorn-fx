#include "PopcornPlayer.h"
#include "AppProperties.h"
#include "VideoPlayer.h"
#include <QDesktopWidget>
#include <QObject>
#include <QtCore/QCoreApplication>
#include <QtQml/QQmlApplicationEngine>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStackedLayout>
#include <iostream>
#include <regex>

using namespace std;

PopcornPlayer::PopcornPlayer(int &argc, char **argv) : argc(argc) {
    this->argv = argv;
    this->app = nullptr;
    this->window = nullptr;
}

int PopcornPlayer::exec() {
    try {
        cout << "Initializing Popcorn PopcornPlayer application" << endl;
        QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
        QCoreApplication::setApplicationName(ApplicationTitle);
        QApplication application(argc, argv);
        this->app = &application;

        cout << "Initializing Popcorn PopcornPlayer player" << endl;
        window = new PopcornPlayerWindow();

        // hide mouse
        QCursor cursor(Qt::BlankCursor);
        QApplication::setOverrideCursor(cursor);
        QApplication::changeOverrideCursor(cursor);

        // set the initialize base size of the player
        cout << "Popcorn PopcornPlayer initialized" << endl;

        int exit = QApplication::exec();
        cout << "Popcorn PopcornPlayer finished with " + std::to_string(exit) << endl;
        return exit;
    } catch (std::exception &ex) {
        cerr << "Popcorn PopcornPlayer execution failed " << ex.what() << endl;
        return -1;
    }
}

bool PopcornPlayer::isInitialized() {
    // check if the app & player have been assigned
    return this->app != nullptr && this->window->player != nullptr;
}

void PopcornPlayer::show() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    invokeOnQt([&] {
        cout << "Showing Popcorn PopcornPlayer" << endl;
        window->showNormal();
    });
}

void PopcornPlayer::showMaximized() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    invokeOnQt([&] {
        cout << "Showing Popcorn PopcornPlayer as maximized" << endl;
        auto *desktop = QApplication::desktop();

        window->setMinimumSize(desktop->geometry().size());
        window->showMaximized();
    });
}

void PopcornPlayer::close() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }

    invokeOnQt([&] {
        window->close();
        QApplication::exit(0);
    });
}

void PopcornPlayer::play(const char *mrl) {
    if (mrl == nullptr) {
        cerr << "No MRL has been passed to the play function, ignoring play action"
             << endl;
        return;
    }

    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }

    invokeOnQt([&, mrl] {
        if (isHttpUrl(mrl)) {
            this->window->player->playUrl(mrl);
        } else {
            this->window->player->playFile(mrl);
        }
    });
}

void PopcornPlayer::pause() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    this->window->player->pause();
}

void PopcornPlayer::resume() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    this->window->player->resume();
}

void PopcornPlayer::stop() {
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    invokeOnQt([&] {
        this->window->player->stop();
        window->hide();
    });
}

bool PopcornPlayer::isMaximized() { return window->isMaximized(); }

void PopcornPlayer::setMaximized(bool maximized) {
    if (maximized) {
        window->showMaximized();
    } else {
        window->showNormal();
    }
}

bool PopcornPlayer::isHttpUrl(const char *url) {
    std::string value = url;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}

template<typename Func>
void PopcornPlayer::invokeOnQt(Func func) {
#if defined(Q_OS_WIN)
    QMetaObject::invokeMethod(this->app, [&] {
#endif
    try {
        func();
    } catch (std::exception &ex) {
        cerr << "Qt invocation failed, " << ex.what() << endl;
    }
#if defined(Q_OS_WIN)
    });
#endif
}
