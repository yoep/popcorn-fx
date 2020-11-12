#include "PopcornPlayer.h"
#include "AppProperties.h"
#include "VideoPlayer.h"
#include <QDesktopWidget>
#include <QObject>
#include <QtCore/QCoreApplication>
#include <QtGui/QFontDatabase>
#include <QtQml/QQmlApplicationEngine>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStackedLayout>
#include <iostream>
#include <regex>

using namespace std;

PopcornPlayer::PopcornPlayer(int& argc, char** argv)
    : argc(argc)
{
    this->argv = argv;
    this->app = nullptr;
    this->window = nullptr;
    this->log = Log::getInstance();
}

PopcornPlayer::~PopcornPlayer()
{
    log->debug("Releasing Popcorn Player resources");
    delete (log);
}

int PopcornPlayer::exec()
{
    try {
        log->trace("Initializing Popcorn Player application");
        QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
        QCoreApplication::setApplicationName(ApplicationTitle);
        QApplication application(argc, argv);
        this->app = &application;

        // set the icon of the window
        loadIcon();

        // load the fonts used by this application
        loadFonts();

        log->trace("Initializing Popcorn Player");
        window = new PopcornPlayerWindow();

        // hide mouse
        QCursor cursor(Qt::BlankCursor);
        QApplication::setOverrideCursor(cursor);
        QApplication::changeOverrideCursor(cursor);

        // set the initialize base size of the player
        log->debug("Popcorn Player initialized");

        int exit = QApplication::exec();
        log->info(std::string("Finished with status " + std::to_string(exit)));
        return exit;
    } catch (std::exception& ex) {
        log->error("Popcorn Player execution failed ", ex);
        return -1;
    }
}

bool PopcornPlayer::isInitialized()
{
    // check if the app & player have been assigned
    return this->app != nullptr && this->window->player != nullptr;
}

void PopcornPlayer::show()
{
    if (this->app == nullptr) {
        log->error(ApplicationNotInitialized);
        return;
    }
    if (window == nullptr) {
        log->error(WindowNotInitialized);
        return;
    }

    invokeOnQt([&] {
        log->debug("Showing Popcorn Player");
        window->showNormal();
    });
}

void PopcornPlayer::showMaximized()
{
    if (this->app == nullptr) {
        cerr << ApplicationNotInitialized << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WindowNotInitialized << endl;
        return;
    }

    invokeOnQt([&] {
        log->debug("Showing Popcorn Player as maximized");
        auto* desktop = QApplication::desktop();

        window->setMinimumSize(desktop->geometry().size());
        window->showMaximized();
    });
}

void PopcornPlayer::close()
{
    if (this->app == nullptr) {
        log->error(ApplicationNotInitialized);
        return;
    }

    invokeOnQt([&] {
        window->close();
        QApplication::exit(0);
    });
}

void PopcornPlayer::play(const char* mrl)
{
    if (mrl == nullptr) {
        log->error("No MRL has been passed to the play function, ignoring play action");
        return;
    }

    if (this->app == nullptr) {
        log->error(ApplicationNotInitialized);
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

void PopcornPlayer::pause()
{
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

void PopcornPlayer::resume()
{
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

void PopcornPlayer::stop()
{
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

void PopcornPlayer::setMaximized(bool maximized)
{
    if (maximized) {
        window->showMaximized();
    } else {
        window->showNormal();
    }
}

template <typename Func>
void PopcornPlayer::invokeOnQt(Func func)
{
    QMetaObject::invokeMethod(this->app, [this, func] {
        try {
            func();
        } catch (std::exception& ex) {
            log->error("Qt invocation failed", ex);
        }
    });
}

void PopcornPlayer::loadIcon()
{
    QApplication::setWindowIcon(QIcon(":/images/icon.png"));
}

void PopcornPlayer::loadFonts()
{
    log->trace("Loading custom fonts");
    if (QFontDatabase::addApplicationFont(":/fonts/FontAwesomeRegular.ttf") == -1) {
        log->warn("Failed to load font FontAwesomeRegular.ttf");
    }
    if (QFontDatabase::addApplicationFont(":/fonts/FontAwesomeSolid.ttf") == -1) {
        log->warn("Failed to load font FontAwesomeSolid.ttf");
    }
    if (QFontDatabase::addApplicationFont(":/fonts/OpenSansBold.ttf") == -1) {
        log->warn("Failed to load font OpenSansBold.ttf");
    }
    if (QFontDatabase::addApplicationFont(":/fonts/OpenSansRegular.ttf") == -1) {
        log->warn("Failed to load font OpenSansRegular.ttf");
    }
    if (QFontDatabase::addApplicationFont(":/fonts/OpenSansSemibold.ttf") == -1) {
        log->warn("Failed to load font OpenSansSemibold.ttf");
    }
    log->debug("Fonts have been loaded");
}

bool PopcornPlayer::isHttpUrl(const char* url)
{
    std::string value = url;
    return std::regex_match(value, std::regex("^(https?:\\/\\/).*"));
}
