#include "PopcornPlayer.h"

#include "AppProperties.h"
#include "QApplicationManager.h"
#include "widgets/VideoWidget.h"

#include <QDesktopWidget>
#include <QtConcurrent/QtConcurrent>
#include <QtGui/QFontDatabase>
#include <QtQml/QQmlApplicationEngine>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStackedLayout>
#include <getopt.h>
#include <iostream>
#include <memory>
#include <player/MediaPlayerFactory.h>

using namespace std;

PopcornPlayer::PopcornPlayer(int &argc, char **argv)
    : _argc(argc)
{
    this->_log = Log::instance();
    this->_log->setApplicationName(APPLICATION_TITLE);

    this->_log->info("Popcorn Player is being started");
    this->_argv = argv;
    this->_window = nullptr;
    this->_mediaPlayer = nullptr;
    this->_eventManager = nullptr;

    // check if we need to parse program arguments
    if (argc > 0 && argv != nullptr) {
        parseArguments();
    }

    init();
}

PopcornPlayer::~PopcornPlayer()
{
    _log->debug("Releasing Popcorn Player resources");

    // release the fonts from Qt
    try {
        QFontDatabase::removeApplicationFont(this->_fontAwesomeRegularId);
        QFontDatabase::removeApplicationFont(this->_fontAwesomeSolidId);
        QFontDatabase::removeApplicationFont(this->_openSansBoldId);
        QFontDatabase::removeApplicationFont(this->_openSansRegularId);
        QFontDatabase::removeApplicationFont(this->_openSansSemiBoldId);
    } catch (std::exception &ex) {
        _log->error(std::string("Failed to release fonts, ") + ex.what(), ex);
    }

    // dispose media player/item resources
    this->_mediaPlayer.reset();
    this->_eventManager.reset();

    // do not dispose the media player factory before the media player is disposed
    // otherwise, the VLC instance will already have been freed before any media items/media players can be freed
    MediaPlayerFactory::dispose();
}

void PopcornPlayer::init()
{
    QApplicationManager::instance()->runInQt(new QLambda([this]() {
        _log->trace("Initializing Popcorn Player application");

        // set the icon of the window
        loadIcon();

        // load the fonts used by this application
        loadFonts();

        _log->trace("Initializing Popcorn Player");
        _window = std::make_shared<PopcornPlayerWindow>();

        // initialize the media player & event manager
        _mediaPlayer = std::shared_ptr<MediaPlayer>(MediaPlayerFactory::createPlayer());
        _eventManager = std::make_shared<PopcornPlayerEventManager>(_mediaPlayer.get());

        // connect the events
        _window->connectMediaPlayerEvents(_mediaPlayer.get());

        // add the video surface to the media player
        _mediaPlayer->setVideoSurface(_window->requestVideoSurface());

        QObject::connect(QApplicationManager::instance()->application(), &QCoreApplication::aboutToQuit,
            [this] {
                stop();
            });

        // set the initialize base size of the player
        _log->debug("Popcorn Player initialized");
    }));
}

void PopcornPlayer::show()
{
    QApplicationManager::instance()->runInQt(new QLambda([this]() {
        if (_window == nullptr) {
            _log->error(WINDOW_NOT_INITIALIZED);
            return;
        }

        _log->debug("Showing Popcorn Player");
        _window->showMaximized();
    }));
}

void PopcornPlayer::setFullscreen(bool fullscreen)
{
    QApplicationManager::instance()->runInQt(new QLambda([this, fullscreen]() {
        if (_window == nullptr) {
            _log->error(WINDOW_NOT_INITIALIZED);
            return;
        }

        if (fullscreen) {
            _log->debug("Showing Popcorn Player in fullscreen mode");
            _window->showFullScreen();
        } else {
            show();
        }
    }));
}

void PopcornPlayer::close()
{
    QApplicationManager::instance()->runInQt(new QLambda([this]() {
        _window->close();
        QApplication::quit();
    }));
}

void PopcornPlayer::play(const char *mrl)
{
    auto *media = MediaPlayerFactory::createMedia(mrl);

    QApplicationManager::instance()->runInQt(new QLambda([this, media]() {
        _mediaPlayer->play(media);
    }));
}

void PopcornPlayer::seek(long time)
{
    _mediaPlayer->seek(time);
}

void PopcornPlayer::pause()
{
    _mediaPlayer->pause();
}

void PopcornPlayer::resume()
{
    _mediaPlayer->resume();
}

void PopcornPlayer::stop()
{
    if (_mediaPlayer == nullptr)
        return;

    QApplicationManager::instance()->runInQt(new QLambda([this]() {
        _mediaPlayer->stop();

        if (_window == nullptr) {
            this->_log->error(WINDOW_NOT_INITIALIZED);
            return;
        }

        _window->hide();
    }));
}

void PopcornPlayer::setSubtitleFile(const char *uri)
{
    _mediaPlayer->setSubtitleFile(uri);
}

void PopcornPlayer::setSubtitleDelay(long delay)
{
    _mediaPlayer->setSubtitleDelay(delay);
}

void PopcornPlayer::registerStateCallback(popcorn_player_state_callback_t callback)
{
    if (waitForEventManager()) {
        _eventManager->addStateCallback(callback);
    }
}

void PopcornPlayer::registerTimeCallback(popcorn_player_time_callback_t callback)
{
    if (waitForEventManager()) {
        _eventManager->addTimeCallback(callback);
    }
}

void PopcornPlayer::registerDurationCallback(popcorn_player_duration_callback_t callback)
{
    if (waitForEventManager()) {
        _eventManager->addDurationCallback(callback);
    }
}

void PopcornPlayer::parseArguments()
{
    int arg;
    while ((arg = getopt(_argc, _argv, "l:h")) != -1) {
        if (arg == 'h') {
            cout << APPLICATION_TITLE << " usage: libPopcornPlayer <options> <mrl>" << endl;
            cout << "Options:" << endl;
            cout << "\t-l <level>\tSet the log level (trace, debug, info, warn, error)" << endl;
            cout << "\t-h\t\t\tShow this help message" << endl;
        }
    }

    auto *result = (logLevel::LogLevel *)malloc(sizeof(enum logLevel::LogLevel));
    Log::parseLogLevel(_argc, _argv, result);

    if (result != nullptr) {
        this->_log->setLevel(*result);
    }

    // free the allocated memory for the log level
    free(result);
}

bool PopcornPlayer::waitForEventManager()
{
    const auto startTime = std::chrono::system_clock::now();

    // block the calling thread until the event manager is initialized
    while (_eventManager == nullptr) {
        const auto currentTime = chrono::system_clock::now();
        auto elapsedTime = std::chrono::duration_cast<std::chrono::seconds>(currentTime - startTime).count();

        // check if we're not waiting indefinitely
        if (elapsedTime > 5) {
            _log->error("Failed to wait for event manager condition");
            return false;
        }

        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }

    return true;
}

void PopcornPlayer::loadIcon()
{
    QApplication::setWindowIcon(QIcon(":/images/icon.png"));
}

void PopcornPlayer::loadFonts()
{
    _log->trace("Loading custom fonts");

    _fontAwesomeRegularId = QFontDatabase::addApplicationFont(":/fonts/FontAwesomeRegular.ttf");
    if (_fontAwesomeRegularId == -1) {
        _log->warn("Failed to load font FontAwesomeRegular.ttf");
    }

    _fontAwesomeSolidId = QFontDatabase::addApplicationFont(":/fonts/FontAwesomeSolid.ttf");
    if (_fontAwesomeSolidId == -1) {
        _log->warn("Failed to load font FontAwesomeSolid.ttf");
    }

    _openSansBoldId = QFontDatabase::addApplicationFont(":/fonts/OpenSansBold.ttf");
    if (_openSansBoldId == -1) {
        _log->warn("Failed to load font OpenSansBold.ttf");
    }

    _openSansRegularId = QFontDatabase::addApplicationFont(":/fonts/OpenSansRegular.ttf");
    if (_openSansRegularId == -1) {
        _log->warn("Failed to load font OpenSansRegular.ttf");
    }

    _openSansSemiBoldId = QFontDatabase::addApplicationFont(":/fonts/OpenSansSemibold.ttf");
    if (_openSansSemiBoldId == -1) {
        _log->warn("Failed to load font OpenSansSemibold.ttf");
    }

    _log->debug("Fonts have been loaded");
}
