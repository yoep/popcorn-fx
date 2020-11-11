#include "PopcornPlayerWindow.h"
#include "./ui_popcornplayerwindow.h"

PopcornPlayerWindow::PopcornPlayerWindow(QWidget *parent) : QMainWindow(parent), ui(new Ui::PopcornPlayerWindow) {
    ui->setupUi(this);

    this->player = ui->player;
}

PopcornPlayerWindow::~PopcornPlayerWindow() {
    delete ui;
}
