#ifndef POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
#define POPCORN_PLAYER_POPCORNPLAYERWINDOW_H

#include <QMainWindow>
#include "VideoPlayer.h"

QT_BEGIN_NAMESPACE
namespace Ui { class PopcornPlayerWindow; }
QT_END_NAMESPACE

class PopcornPlayerWindow : public QMainWindow {
Q_OBJECT

public:
    PopcornPlayerWindow(QWidget *parent = nullptr);

    ~PopcornPlayerWindow();

    VideoPlayer *player;

private:
    Ui::PopcornPlayerWindow *ui;
};

#endif // POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
