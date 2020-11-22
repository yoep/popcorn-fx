#include "PopcornPlayerWindow.h"

#include "PlayerHeader.h"
#include "ui_popcornplayerwindow.h"

#include <QResizeEvent>
#include <QtWidgets/QGridLayout>

PopcornPlayerWindow::PopcornPlayerWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::PopcornPlayerWindow)
{
    initializeUi();
}

PopcornPlayerWindow::~PopcornPlayerWindow()
{
    delete (ui);
}

WId PopcornPlayerWindow::requestVideoSurface()
{
    return ui->player->request();
}

void PopcornPlayerWindow::releaseVideoSurface()
{
    ui->player->release();
}

void PopcornPlayerWindow::showEvent(QShowEvent *event)
{
    QWidget::showEvent(event);
}

void PopcornPlayerWindow::initializeUi()
{
    ui->setupUi(this);

    ui->rootLayout->setRowStretch(1, QLAYOUTSIZE_MAX);
    ui->rootLayout->setRowMinimumHeight(3, 75);
}
