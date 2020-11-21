#include "VideoWidget.h"

#include "Log.h"
#include "PlayerControls.h"

#include <QtWidgets/QStackedLayout>

using namespace std;

//region Constructors

VideoWidget::VideoWidget(QWidget *parent)
    : QFrame(parent)
{
    this->log = Log::getInstance();
    this->layout = new QStackedLayout(this);
    this->videoSurface = nullptr;

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
    if (videoSurface) {
        log->warn("Video surface is already in use, release it before requesting a new one");
        return -1;
    }

    videoSurface = new QWidget();

    videoSurface->setContextMenuPolicy(Qt::PreventContextMenu);
    QPalette plt = palette();
    plt.setColor(QPalette::Window, Qt::black);
    videoSurface->setPalette(plt);
    videoSurface->setAutoFillBackground(true);
    // Force the widget to be native so that it gets a winId()
    videoSurface->setAttribute(Qt::WA_NativeWindow, true);

#if !defined(QT5_HAS_X11)
    videoSurface->setAttribute(Qt::WA_PaintOnScreen, true);
#else
    videoSurface->setMouseTracking(true);
    this->setMouseTracking(true);
#endif

    layout->addWidget(videoSurface);

    sync();
    return videoSurface->winId();
}

void VideoWidget::release()
{
    if (videoSurface) {
        log->trace("Video surface is being released");
        layout->removeWidget(videoSurface);
        videoSurface->deleteLater();
        videoSurface = nullptr;
        log->debug("Video surface released");
    }
}

//endregion

//region Functions

void VideoWidget::initializeUi()
{
    log->trace("Initializing video widget");
    // update the layout
    layout->setContentsMargins(0, 0, 0, 0);

    // set the pallet of this widget
    QPalette plt = palette();
    plt.setColor(QPalette::Window, Qt::black);
    this->setPalette(plt);
    log->debug("Video widget initialized");
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
