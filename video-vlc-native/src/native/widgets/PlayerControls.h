#ifndef POPCORNPLAYER_PLAYERCONTROLS_H
#define POPCORNPLAYER_PLAYERCONTROLS_H

#include <Log.h>
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
    /**
     * Set the new time value of the current media playback.
     *
     * @param newValue The new time value.
     */
    void setTime(long newValue);

    /**
     * Set the new duration value of the current media playback.
     *
     * @param newValue The new duration value.
     */
    void setDuration(long newValue);

private:
    Ui::PlayerControls *ui;
    Log *log;

    void initializeUi();

protected:
    void paintEvent(QPaintEvent *event) override;
};

#endif //POPCORNPLAYER_PLAYERCONTROLS_H
