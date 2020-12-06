#ifndef POPCORNPLAYER_PLAYERCONTROLS_H
#define POPCORNPLAYER_PLAYERCONTROLS_H

#include <Log.h>
#include <QtWidgets/QFrame>
#include <player/MediaPlayerState.h>

QT_BEGIN_NAMESPACE
namespace Ui {
class PlayerControls;
}
QT_END_NAMESPACE

class PlayerControls : public QFrame {
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

    /**
     * Set the new media player state.
     *
     * @param newValue The new state of the media player.
     */
    void setPlayerState(MediaPlayerState newValue);

private slots:
    /**
     * Invoked when the stop button is invoked.
     */
    void onStop();

    /**
     * Invoked when the backward button is invoked.
     */
    void onBackward();

    /**
     * Invoked when the play pause is invoked.
     */
    void onPlayPause();

    /**
     * Invoked when the forward button is invoked.
     */
    void onForward();

signals:
    /**
     * Signals that the stop button has been invoked.
     */
    void stop();

    /**
     * Signals that the backward button has been invoked.
     */
    void backward();

    /**
     * Signals that the play/pause button has been invoked.
     */
    void playPause();

    /**
     * Signals that the forward button has been invoked.
     */
    void forward();

private:
    Ui::PlayerControls *ui;
    Log *log;

    void initializeUi();

protected:
    void keyPressEvent(QKeyEvent *event) override;

    /**
     * Convert the given time value to an applicable slider value.
     *
     * @param value The time value to convert.
     * @return Returns the converted value for the slider.
     */
    static int toSliderValue(long value);
};

#endif //POPCORNPLAYER_PLAYERCONTROLS_H
