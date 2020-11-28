#ifndef POPCORNPLAYER_QLAMBDAEVENT_H
#define POPCORNPLAYER_QLAMBDAEVENT_H

#include "QLamba.h"

#include <QtCore/QEvent>

using namespace qlambda;

/**
 * The QLambdaEvent type identifier.
 */
const int Q_LAMBDA_EVENT_TYPE = 49222;

/**
 * A lambda event which is ran within the QApplication main event loop.
 */
class QLambdaEvent : public QEvent {
public:
    QLambdaEvent(AbstractQLambda *runnable)
        : QEvent(QEvent::Type(Q_LAMBDA_EVENT_TYPE))
    {
        _runnable = runnable;
    }

    ~QLambdaEvent() override
    {
        if (_runnable != nullptr) {
            _runnable->run();
        }
        delete _runnable;
    }

private:
    AbstractQLambda *_runnable = nullptr;
};

#endif //POPCORNPLAYER_QLAMBDAEVENT_H
