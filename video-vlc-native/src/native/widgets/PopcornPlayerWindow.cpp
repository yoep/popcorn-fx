#include "PopcornPlayerWindow.h"

#include "PlayerHeader.h"
#include "ui_popcornplayerwindow.h"

#include <QResizeEvent>
#include <QTimer>
#include <QtWidgets/QGridLayout>
#include <player/MediaPlayer.h>

PopcornPlayerWindow::PopcornPlayerWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::PopcornPlayerWindow)
{
    this->_log = Log::instance();
    this->_fadeTimer = new QTimer(this);

    initializeUi();
    connectEvents();
}

PopcornPlayerWindow::~PopcornPlayerWindow()
{
    releaseVideoSurface();

    delete ui;
}

WId PopcornPlayerWindow::requestVideoSurface()
{
    return ui->player->request();
}

void PopcornPlayerWindow::releaseVideoSurface()
{
    ui->player->release();
}

void PopcornPlayerWindow::connectMediaPlayerEvents(MediaPlayer *mediaPlayer)
{
    _log->trace("Connecting media player signals");
    connect(mediaPlayer, &MediaPlayer::timeChanged,
        ui->controls, &PlayerControls::setTime);
    connect(mediaPlayer, &MediaPlayer::durationChanged,
        ui->controls, &PlayerControls::setDuration);
    connect(mediaPlayer, &MediaPlayer::stateChanged,
        ui->controls, &PlayerControls::setPlayerState);
    connect(mediaPlayer, &MediaPlayer::stateChanged,
        this, &PopcornPlayerWindow::onStateChanged);
}

void PopcornPlayerWindow::hideUi()
{
    _log->trace("Hiding popcorn player window UI");
    ui->controls->hide();
}

void PopcornPlayerWindow::onStateChanged(MediaPlayerState newState)
{
    if (newState == PLAYING) {
        _fadeTimer->start();
    } else if (newState == PAUSED) {
        ui->controls->show();
    }
}

void PopcornPlayerWindow::initializeUi()
{
    _log->trace("Initializing popcorn player window");
    ui->setupUi(this);

    setAutoFillBackground(true);

    ui->rootLayout->setRowStretch(1, QLAYOUTSIZE_MAX);
    ui->rootLayout->setRowMinimumHeight(3, 75);

    _fadeTimer->setInterval(2000);
    _fadeTimer->setSingleShot(true);
    _log->debug("Popcorn player window initialized");
}

void PopcornPlayerWindow::connectEvents()
{
    _log->trace("Connecting popcorn player window slots");
    connect(_fadeTimer, &QTimer::timeout,
        this, &PopcornPlayerWindow::hideUi);
    _log->debug("Popcorn player window slots have been connected");
}
