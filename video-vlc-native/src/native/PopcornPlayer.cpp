#include "PopcornPlayer.h"

#include "AppProperties.h"
#include "widgets/VideoWidget.h"

#include <QDesktopWidget>
#include <QtGui/QFontDatabase>
#include <QtQml/QQmlApplicationEngine>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStackedLayout>
#include <getopt.h>
#include <iostream>
#include <player/MediaPlayerFactory.h>
#include <regex>

using namespace std;

PopcornPlayer::PopcornPlayer(int &argc, char **argv)
    : argc(argc)
{
    this->log = Log::getInstance();
    this->argv = argv;
    this->app = nullptr;
    this->window = nullptr;
    this->mediaPlayer = MediaPlayerFactory::create();

    // check if we need to parse program arguments
    if (argc > 0 && argv != nullptr) {
        parseArguments();
    }
}

PopcornPlayer::~PopcornPlayer()
{
    log->debug("Releasing Popcorn Player resources");

    if (this->app != nullptr) {
        log->trace("Quiting the QT application");
        QApplication::quit();
    }

    delete (mediaPlayer);
    delete (log);
}

int PopcornPlayer::exec()
{
    Q_INIT_RESOURCE(PopcornPlayer);

    try {
        log->trace("Initializing Popcorn Player application");
        QApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
        QApplication::setAttribute(Qt::AA_UseHighDpiPixmaps);
        QApplication::setApplicationName(APPLICATION_TITLE);
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

        // stop the player in case it might be still playing
        mediaPlayer->stop();

        return exit;
    } catch (std::exception &ex) {
        log->error("Popcorn Player execution failed ", ex);
        return -1;
    }
}

void PopcornPlayer::show()
{
    if (this->app == nullptr) {
        log->error(APPLICATION_NOT_INITIALIZED);
        return;
    }
    if (window == nullptr) {
        log->error(WINDOW_NOT_INITIALIZED);
        return;
    }

    invokeOnQt([&] {
        log->debug("Showing Popcorn Player");
        window->showMaximized();
    });
}

void PopcornPlayer::setFullscreen(bool fullscreen)
{
    if (this->app == nullptr) {
        log->error(APPLICATION_NOT_INITIALIZED);
        return;
    }
    if (window == nullptr) {
        log->error(WINDOW_NOT_INITIALIZED);
        return;
    }

    invokeOnQt([&, fullscreen] {
        if (fullscreen) {
            log->debug("Showing Popcorn Player in fullscreen mode");
            window->showFullScreen();
        } else {
            show();
        }
    });
}

void PopcornPlayer::close()
{
    if (this->app == nullptr) {
        log->error(APPLICATION_NOT_INITIALIZED);
        return;
    }

    invokeOnQt([&] {
        stop();
        window->close();
        QApplication::quit();
    });
}

void PopcornPlayer::play(const char *mrl)
{
    // add the video surface to the media player
    mediaPlayer->setVideoSurface(window->requestVideoSurface());
    mediaPlayer->play(mrl);
}

void PopcornPlayer::pause()
{
    mediaPlayer->pause();
}

void PopcornPlayer::resume()
{
    mediaPlayer->resume();
}

void PopcornPlayer::stop()
{
    if (this->app == nullptr) {
        cerr << APPLICATION_NOT_INITIALIZED << endl;
        return;
    }
    if (window == nullptr) {
        cerr << WINDOW_NOT_INITIALIZED << endl;
        return;
    }

    invokeOnQt([&] {
        mediaPlayer->stop();
        window->releaseVideoSurface();
        window->hide();
    });
}

void PopcornPlayer::setSubtitleFile(const char *uri)
{
    if (this->app == nullptr) {
        cerr << APPLICATION_NOT_INITIALIZED << endl;
        return;
    }

    mediaPlayer->setSubtitleFile(uri);
}

void PopcornPlayer::setSubtitleDelay(long delay)
{
    if (this->app == nullptr) {
        cerr << APPLICATION_NOT_INITIALIZED << endl;
        return;
    }

    mediaPlayer->setSubtitleDelay(delay);
}

template <typename Func>
void PopcornPlayer::invokeOnQt(Func func)
{
    QMetaObject::invokeMethod(this->app, [this, func] {
        try {
            func();
        } catch (std::exception &ex) {
            log->error("Qt invocation failed", ex);
        }
    });
}

void PopcornPlayer::parseArguments()
{
    int arg;
    while ((arg = getopt(argc, argv, "l:h")) != -1) {
        switch (arg) {
        case 'l':
            if (optarg) {
                updateLogLevel(optarg);
            }
            break;
        case 'h':
        default:
            cout << APPLICATION_TITLE << " usage: libPopcornPlayer <options> <mrl>" << endl;
            cout << "Options:" << endl;
            cout << "\t-l <level>\tSet the log level (trace, debug, info, warn, error)" << endl;
            cout << "\t-h\t\t\tShow this help message" << endl;
            break;
        }
    }
}

void PopcornPlayer::updateLogLevel(char *levelArg)
{
    // put the level to lower case
    for (int i = 0; i < strlen(levelArg); i++) {
        levelArg[i] = std::tolower(levelArg[i]);
    }

    std::string level(levelArg);

    if (level == "trace") {
        this->log->setLevel(TRACE);
    } else if (level == "debug") {
        this->log->setLevel(DEBUG);
    } else if (level == "info") {
        this->log->setLevel(INFO);
    } else if (level == "warn") {
        this->log->setLevel(WARN);
    } else if (level == "error") {
        this->log->setLevel(ERROR);
    }
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
