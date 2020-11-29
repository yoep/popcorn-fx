#include "QApplicationManager.h"

#include "AppProperties.h"
#include "QLambdaEvent.h"

#include <QtConcurrent/QtConcurrent>
#include <QtCore/QCoreApplication>
#include <QtWidgets/QApplication>
#include <widgets/PopcornPlayerWindow.h>

using namespace std;

shared_ptr<QApplicationManager> QApplicationManager::_instance = nullptr;

QApplicationManager::QApplicationManager()
{
    this->_log = Log::instance();
    this->_qtApp = nullptr;

    initialize();
}

QApplicationManager::~QApplicationManager()
{
    _log->debug("Disposing the QApplicationManager");
    QApplication::quit();

    if (_qtThread.joinable()) {
        _qtThread.join();
    }
}

QApplicationManager *QApplicationManager::instance()
{
    if (_instance == nullptr) {
        _instance = std::make_shared<QApplicationManager>();
    }

    return _instance.get();
}

bool QApplicationManager::isFinished()
{
    return _qtAppFinished;
}

bool QApplicationManager::isRunning()
{
    return _running;
}

QCoreApplication *QApplicationManager::application()
{
    return _qtApp;
}

void QApplicationManager::runInQt(AbstractQLambda *runnable)
{
    if (isFinished()) {
        _log->error("Unable to execute QLambda, application is in invalid state \"finished\"");
        return;
    }

    while (!isRunning())
        std::this_thread::sleep_for(std::chrono::milliseconds(50));

    QCoreApplication::postEvent(_qtApp, new QLambdaEvent(runnable));
}

void QApplicationManager::initialize()
{
    _log->trace("Initializing QApplication manager");
    if (QCoreApplication::instance() != nullptr) {
        _log->error("A QT application instance already exists");
        _log->error("QApplicationManager will not be initialized!");
        return;
    }

    _log->debug("Updating environment");
    // disable the GLIB event loop as it crashes when this library is launched through JNA
    // if we don't do this, the exec will get stuck on "g_main_context_push_thread_default: assertion 'acquired_context' failed"
    _log->trace("Disabling the GLIB event loop");
    putenv("QT_NO_GLIB=1");

    _log->trace("Initializing new QT thread");
    _qtThread = std::thread([&]() {
        auto argc = 1;
        auto arg = std::string("PopcornPlayer");
        char *argv[1] = { arg.data() };

        Q_INIT_RESOURCE(PopcornPlayer);

        _log->trace("Initializing QT application instance");
        QApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
        QApplication::setApplicationName(APPLICATION_TITLE);
        _qtApp = new QApplication(argc, argv);

        QObject::connect(_qtApp, &QApplication::aboutToQuit, _qtApp, [this]() {
            _qtAppFinished = true;
        });

        QApplication::setOverrideCursor(Qt::BlankCursor);

        _log->debug("Starting application");
        _running = true;
        int state = _qtApp->exec();

        _log->info("QT application finished with state " + std::to_string(state));
        _running = false;
    });
}
