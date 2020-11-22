#ifndef POPCORNPLAYER_PLAYERCONTROLS_H
#define POPCORNPLAYER_PLAYERCONTROLS_H

#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE
namespace Ui {
class PlayerControls;
}
QT_END_NAMESPACE

class PlayerControls : public QWidget {
    Q_OBJECT
public:
    PlayerControls(QWidget *parent = nullptr);

    ~PlayerControls();

public slots:


private:
    Ui::PlayerControls *ui;

    void initializeUi();
};

#endif //POPCORNPLAYER_PLAYERCONTROLS_H
