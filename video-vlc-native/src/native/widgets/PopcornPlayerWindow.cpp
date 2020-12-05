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

    connect(ui->controls, &PlayerControls::stop,
        mediaPlayer, [mediaPlayer] {
            mediaPlayer->stop();
        });
    connect(ui->controls, &PlayerControls::backward,
        mediaPlayer, [&] {
            updateTime(-5000);
        });
    connect(ui->controls, &PlayerControls::playPause,
        mediaPlayer, [&] {
            togglePlayback();
        });
    connect(ui->controls, &PlayerControls::forward,
        mediaPlayer, [&] {
            updateTime(5000);
        });

    // store the media player reference
    this->_mediaPlayer = mediaPlayer;
}

void PopcornPlayerWindow::onHideUI()
{
    hideOverlay();
}

void PopcornPlayerWindow::onStateChanged(MediaPlayerState newState)
{
    if (newState == MediaPlayerState::PLAYING) {
        _fadeTimer->start();
    } else if (newState == MediaPlayerState::PAUSED) {
        _fadeTimer->stop();
        showOverlay();
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
        this, &PopcornPlayerWindow::onHideUI);
    _log->debug("Popcorn player window slots have been connected");
}

void PopcornPlayerWindow::keyPressEvent(QKeyEvent *event)
{
    showOverlay();

    if (_mediaPlayer->state() != MediaPlayerState::PAUSED) {
        _fadeTimer->start();
    }

    QWidget::keyPressEvent(event);
}

void PopcornPlayerWindow::showOverlay()
{
    _log->trace("Showing UI player overlay");
    ui->controls->show();
}

void PopcornPlayerWindow::hideOverlay()
{
    _log->trace("Hiding UI player overlay");
    ui->controls->hide();
}

void PopcornPlayerWindow::togglePlayback()
{
    auto state = _mediaPlayer->state();

    if (state == MediaPlayerState::PLAYING) {
        _mediaPlayer->pause();
    } else if (state == MediaPlayerState::PAUSED) {
        _mediaPlayer->resume();
    } else {
        _log->warn(std::string("Unable to toggle the playback, media player is in invalid state ") + media_player_state_as_string(state));
    }
}

void PopcornPlayerWindow::updateTime(long offset)
{
    auto currentTime = _mediaPlayer->time();
    auto duration = _mediaPlayer->duration();
    auto newTime = currentTime + offset;

    if (newTime < 0) {
        _mediaPlayer->seek(0);
    } else if (newTime > duration) {
        _mediaPlayer->seek(duration);
    } else {
        _mediaPlayer->seek(newTime);
    }
}
