#include "PopcornPlayerWindow.h"
#include "./ui_popcornplayerwindow.h"

PopcornPlayerWindow::PopcornPlayerWindow(QWidget* parent)
    : QMainWindow(parent)
    , ui(new Ui::PopcornPlayerWindow)
{
    initializeUi();

    this->player = ui->player;
}

PopcornPlayerWindow::~PopcornPlayerWindow()
{
    delete ui;
}
void PopcornPlayerWindow::initializeUi()
{
    ui->setupUi(this);
}
