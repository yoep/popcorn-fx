#include "PopcornPlayerWindow.h"

#include "PlayerHeader.h"
#include "ui_popcornplayerwindow.h"

#include <QResizeEvent>
#include <QtWidgets/QGridLayout>
#include <player/Media.h>
#include <player/MediaPlayer.h>

PopcornPlayerWindow::PopcornPlayerWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::PopcornPlayerWindow)
{
    initializeUi();
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

void PopcornPlayerWindow::connectMediaEvents(Media *media)
{
}

void PopcornPlayerWindow::connectMediaPlayerEvents(MediaPlayer *mediaPlayer)
{
    QObject::connect(mediaPlayer, &MediaPlayer::timeChanged,
        ui->controls, &PlayerControls::setTime);
    QObject::connect(mediaPlayer, &MediaPlayer::durationChanged,
        ui->controls, &PlayerControls::setDuration);
    QObject::connect(mediaPlayer, &MediaPlayer::stateChanged,
        ui->controls, &PlayerControls::setPlayerState);
}

void PopcornPlayerWindow::paintEvent(QPaintEvent *event)
{
    QWidget::paintEvent(event);
}

void PopcornPlayerWindow::initializeUi()
{
    ui->setupUi(this);

    setAutoFillBackground(true);

    ui->rootLayout->setRowStretch(1, QLAYOUTSIZE_MAX);
    ui->rootLayout->setRowMinimumHeight(3, 75);
}
