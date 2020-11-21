#include "PopcornPlayerWindow.h"

#include "PlayerHeader.h"

#include <QResizeEvent>

PopcornPlayerWindow::PopcornPlayerWindow(QWidget *parent)
    : QMainWindow(parent)
{
    this->player = new VideoWidget(this);
//    this->header = new PlayerHeader(this);
//    this->controls = new PlayerControls(this);

    initializeUi();
}

PopcornPlayerWindow::~PopcornPlayerWindow()
{
    delete (this->player);
}

WId PopcornPlayerWindow::requestVideoSurface()
{
    return player->request();
}

void PopcornPlayerWindow::releaseVideoSurface()
{
    player->release();
}

void PopcornPlayerWindow::initializeUi()
{
    setCentralWidget(player);
}

void PopcornPlayerWindow::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
}
