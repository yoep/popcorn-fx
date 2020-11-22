#include "PlayerHeader.h"
#include "ui_playerheader.h"
#include <QPainter>
#include <QtWidgets/QStyleOption>

PlayerHeader::PlayerHeader(QWidget *parent)
    : QWidget(parent)
    , ui(new Ui::PlayerHeader)
{
    initializeUi();
}
PlayerHeader::~PlayerHeader()
{
    delete ui;
}

void PlayerHeader::initializeUi()
{
    ui->setupUi(this);

    setAttribute(Qt::WA_NoSystemBackground);
    setAttribute(Qt::WA_TranslucentBackground);
    setAttribute(Qt::WA_PaintOnScreen);
    setStyleSheet("background-color: rgba(0,0,0,0)");
}
