#include "PlayerControls.h"

#include "FontAwesome.h"
#include "ui_playercontrols.h"

#include <QPainter>
#include <QtGui/QFontDatabase>
#include <QtWidgets/QStyleOption>

PlayerControls::PlayerControls(QWidget* parent)
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

    setPalette(Qt::transparent);
    setAttribute(Qt::WA_NoSystemBackground);
    setAttribute(Qt::WA_TranslucentBackground);
    setAttribute(Qt::WA_PaintOnScreen);

    QFont fontAwesome;
    fontAwesome.setFamily(QString("Font Awesome 5 Free"));
    QPalette palette;
    palette.setColor(QPalette::NoRole, QColor(255, 0, 0));

    ui->stopButton->setPalette(palette);
    ui->stopButton->setFont(fontAwesome);
    ui->stopButton->setText(STOP_UNICODE);
    ui->backwardButton->setText(BACKWARD_UNICODE);
    ui->playPauseButton->setText(PLANE_UNICODE);
    ui->forwardButton->setText(FORWARD_UNICODE);
    ui->moreButton->setText(ELLIPSIS_H_UNICODE);
}

void PlayerControls::resizeEvent(QResizeEvent *event)
{
    updatePosition();
    QWidget::resizeEvent(event);
}

void PlayerControls::updatePosition()
{
    move(QPoint(rect().center()));
}

//void PlayerControls::paintEvent(QPaintEvent* event)
//{
//    QStyleOption opt;
//    opt.init(this);
//    QPainter painter(this);
//    style()->drawPrimitive(QStyle::PE_Widget, &opt, &painter, this);
//}
