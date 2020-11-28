#include "PlayerControls.h"

#include "FontAwesome.h"
#include "ui_playercontrols.h"

#include <QPainter>
#include <QtGui/QFontDatabase>
#include <QtWidgets/QStyleOption>

using namespace std;

PlayerControls::PlayerControls(QWidget *parent)
    : QWidget(parent)
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

void PlayerControls::initializeUi()
{
    log->trace("Initializing player controls");
    ui->setupUi(this);

    ui->stopButton->setText(QString(STOP_UNICODE));
    ui->backwardButton->setText(QString(BACKWARD_UNICODE));
    ui->playPauseButton->setText(QString(PLAY_UNICODE));
    ui->forwardButton->setText(QString(FORWARD_UNICODE));
    ui->moreButton->setText(QString(ELLIPSIS_H_UNICODE));
    log->debug("Player controls have been initialized");
}

void PlayerControls::paintEvent(QPaintEvent *event)
{
    QWidget::paintEvent(event);
}
