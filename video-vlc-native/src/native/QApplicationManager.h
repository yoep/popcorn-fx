#ifndef POPCORNPLAYER_QAPPLICATIONMANAGER_H
#define POPCORNPLAYER_QAPPLICATIONMANAGER_H

#include "Log.h"
#include "QLamba.h"

#include <QtCore/QCoreApplication>
#include <memory>
#include <thread>
#include <tuple>

using namespace std;
using namespace qlambda;

class QApplicationManager {

public:
    /**
     * Initialize a new instance of the QApplicationManager.
     * Don't use this constructor yourself as an instance is already available through #instance().
     */
    QApplicationManager();

    ~QApplicationManager();

    /**
     * Get the QT app manager instance.
     *
     * @return Returns the app manager instance.
     */
    static QApplicationManager *instance();

    /**
     * Verify if the QApplication has finished.
     *
     * @return Returns true if the QApplication has finished it's execution, else false.
     */
    bool isFinished();

    /**
     * Verify if the QApplication is running and able to accept events.
     *
     * @return Returns true if the application is running, else false.
     */
    bool isRunning();

    /**
     * Execute/run the given QLambda within the QApplication event loop.
     *
     * @param runnable The lambda to execute.
     * @see QLambda
     */
    void runInQt(AbstractQLambda *runnable);

private:
    static shared_ptr<QApplicationManager> _instance;
    QCoreApplication *_qtApp;
    std::thread _qtThread;
    std::atomic<bool> _qtAppFinished = false;
    std::atomic<bool> _running = false;
    Log *_log;

    void initialize();
};

#endif //POPCORNPLAYER_QAPPLICATIONMANAGER_H
