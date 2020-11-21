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

protected:
    void resizeEvent(QResizeEvent *event) override;

private:
    Ui::PlayerControls *ui;

    void initializeUi();

    void updatePosition();

protected:
    //    void paintEvent(QPaintEvent *event) override;
};

#endif //POPCORNPLAYER_PLAYERCONTROLS_H
