#ifndef POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
#define POPCORN_PLAYER_POPCORNPLAYERWINDOW_H

#include "VideoPlayer.h"
#include <QMainWindow>

QT_BEGIN_NAMESPACE
namespace Ui {
class PopcornPlayerWindow;
}
QT_END_NAMESPACE

class PopcornPlayerWindow : public QMainWindow {
    Q_OBJECT

public:
    PopcornPlayerWindow(QWidget* parent = nullptr);

    ~PopcornPlayerWindow();

    VideoPlayer* player;

private:
    Ui::PopcornPlayerWindow* ui;
    void initializeUi();
};

#endif // POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
