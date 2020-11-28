#include "VideoWidget.h"

#include "Log.h"
#include "PlayerControls.h"

#include <QtWidgets/QStackedLayout>

using namespace std;

//region Constructors

VideoWidget::VideoWidget(QWidget *parent)
    : QFrame(parent)
{
    this->_log = Log::instance();
    this->_layout = new QStackedLayout(this);
    this->_videoSurface = nullptr;

    initializeUi();
}

VideoWidget::~VideoWidget()
{
    release();
}

//endregion

//region Methods

WId VideoWidget::request()
{
    if (_videoSurface) {
        _log->warn("Video surface is already in use, release it before requesting a new one");
        return -1;
    }

    _videoSurface = new QWidget();

    _videoSurface->setContextMenuPolicy(Qt::PreventContextMenu);
    QPalette plt = palette();
    plt.setColor(QPalette::Window, Qt::black);
    _videoSurface->setPalette(plt);
    _videoSurface->setAutoFillBackground(true);
    // Force the widget to be native so that it gets a winId()
    _videoSurface->setAttribute(Qt::WA_NativeWindow, true);

#if !defined(QT5_HAS_X11)
    _videoSurface->setAttribute(Qt::WA_PaintOnScreen, true);
#else
    videoSurface->setMouseTracking(true);
    this->setMouseTracking(true);
#endif

    _layout->addWidget(_videoSurface);

    sync();
    return _videoSurface->winId();
}

void VideoWidget::release()
{
    if (_videoSurface != nullptr) {
        _log->trace("Video surface is being released");
        _layout->removeWidget(_videoSurface);
        _videoSurface->deleteLater();
        _videoSurface = nullptr;
        _log->debug("Video surface released");
    }
}

//endregion

//region Functions

void VideoWidget::initializeUi()
{
    _log->trace("Initializing video widget");
    // update the layout
    _layout->setContentsMargins(0, 0, 0, 0);

    // set the pallet of this widget
    QPalette plt = palette();
    plt.setColor(QPalette::Window, Qt::black);
    this->setPalette(plt);
    _log->debug("Video widget initialized");
}

void VideoWidget::sync()
{
#if defined(Q_WS_X11)
    /* Make sure the X server has processed all requests.
     * This protects other threads using distinct connections from getting
     * the video widget window in an inconsistent states. */
    XSync(QX11Info::display(), False);
#endif
}

//endregion
