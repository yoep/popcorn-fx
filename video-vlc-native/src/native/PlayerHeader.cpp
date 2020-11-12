#include "PlayerHeader.h"
#include "./ui_playerheader.h"

PlayerHeader::PlayerHeader(QWidget* parent)
    : QWidget(parent)
    , ui(new Ui::PlayerHeader)
{
    ui->setupUi(this);
}

PlayerHeader::~PlayerHeader()
{
    delete ui;
}