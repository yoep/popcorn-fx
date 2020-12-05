#include "PlayerControls.h"

#include "FontAwesome.h"
#include "ui_playercontrols.h"

#include <QKeyEvent>
#include <QPainter>
#include <QtGui/QFontDatabase>
#include <QtWidgets/QStyleOption>

using namespace std;

PlayerControls::PlayerControls(QWidget *parent)
    : QFrame(parent)
    , ui(new Ui::PlayerControls)
{
    this->log = Log::instance();

    initializeUi();
}

PlayerControls::~PlayerControls()
{
    delete ui;
}

void PlayerControls::setTime(long newValue)
{
    ui->time->setTime(newValue);
}

void PlayerControls::setDuration(long newValue)
{
    ui->duration->setTime(newValue);
}

void PlayerControls::setPlayerState(MediaPlayerState newValue)
{
    if (newValue == MediaPlayerState::PAUSED) {
        ui->playPauseButton->setText(QString(PLAY_UNICODE));
    } else {
        ui->playPauseButton->setText(QString(PAUSE_UNICODE));
    }
}

void PlayerControls::initializeUi()
{
    log->trace("Initializing player controls");
    ui->setupUi(this);

    // set font awesome unicodes
    ui->stopButton->setText(QString(STOP_UNICODE));
    ui->backwardButton->setText(QString(BACKWARD_UNICODE));
    ui->playPauseButton->setText(QString(PLAY_UNICODE));
    ui->forwardButton->setText(QString(FORWARD_UNICODE));
    ui->moreButton->setText(QString(ELLIPSIS_H_UNICODE));

    // set the icon actions
    connect(ui->stopButton, &Icon::triggerAction,
        this, &PlayerControls::onStop);
    connect(ui->backwardButton, &Icon::triggerAction,
        this, &PlayerControls::onBackward);
    connect(ui->playPauseButton, &Icon::triggerAction,
        this, &PlayerControls::onPlayPause);
    connect(ui->forwardButton, &Icon::triggerAction,
        this, &PlayerControls::onForward);

    // set focus
    ui->stopButton->setFocusPolicy(Qt::FocusPolicy::StrongFocus);
    ui->backwardButton->setFocusPolicy(Qt::FocusPolicy::StrongFocus);
    ui->playPauseButton->setFocusPolicy(Qt::FocusPolicy::StrongFocus);
    ui->forwardButton->setFocusPolicy(Qt::FocusPolicy::StrongFocus);
    ui->moreButton->setFocusPolicy(Qt::FocusPolicy::StrongFocus);
    ui->playPauseButton->setFocus();

    log->debug("Player controls have been initialized");
}

void PlayerControls::keyPressEvent(QKeyEvent *event)
{
    auto key = event->key();

    switch (key) {
    case Qt::Key_Space:
    case Qt::Key_MediaTogglePlayPause:
        onPlayPause();
        break;
    case Qt::Key_MediaPrevious:
    case Qt::Key_Back:
        onBackward();
        break;
    case Qt::Key_MediaNext:
    case Qt::Key_Forward:
        onForward();
        break;
    case Qt::Key_Left: {
        focusPreviousChild();
    } break;
    case Qt::Key_Right: {
        focusNextChild();
    } break;
    default:
        QWidget::keyPressEvent(event);
        break;
    }
}

void PlayerControls::onStop()
{
    emit stop();
}

void PlayerControls::onBackward()
{
    emit backward();
}

void PlayerControls::onPlayPause()
{
    emit playPause();
}

void PlayerControls::onForward()
{
    emit forward();
}
