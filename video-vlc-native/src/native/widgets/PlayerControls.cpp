#include "PlayerControls.h"

#include "FontAwesome.h"
#include "ui_playercontrols.h"

#include <QPainter>
#include <QtGui/QFontDatabase>
#include <QtWidgets/QStyleOption>

PlayerControls::PlayerControls(QWidget *parent)
    : QWidget(parent)
    , ui(new Ui::PlayerControls)
{
    initializeUi();
}

PlayerControls::~PlayerControls()
{
}

void PlayerControls::initializeUi()
{
    ui->setupUi(this);

    QFont fontAwesome = QFont(QString("Font Awesome 5 Free"));
    fontAwesome.setStyleStrategy(QFont::NoFontMerging);
    fontAwesome.setPixelSize(21);

    ui->stopButton->setFont(fontAwesome);
    ui->stopButton->setText(QString(STOP_UNICODE));
    ui->backwardButton->setText(QString(BACKWARD_UNICODE));
    ui->playPauseButton->setText(QString(PLANE_UNICODE));
    ui->forwardButton->setText(QString(FORWARD_UNICODE));
    ui->moreButton->setText(QString(ELLIPSIS_H_UNICODE));
}