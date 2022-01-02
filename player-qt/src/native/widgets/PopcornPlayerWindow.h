#ifndef POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
#define POPCORN_PLAYER_POPCORNPLAYERWINDOW_H

#include "PlayerControls.h"
#include "widgets/VideoWidget.h"

#include <QMainWindow>
#include <player/MediaPlayer.h>

QT_BEGIN_NAMESPACE
namespace Ui {
class PopcornPlayerWindow;
}
QT_END_NAMESPACE

class PopcornPlayerWindow : public QMainWindow {
    Q_OBJECT

public:
    PopcornPlayerWindow(QWidget *parent = nullptr);

    ~PopcornPlayerWindow();

    /**
     * Request the window handle of the video surface.
     * This handle can be used by VLC to render the video on.
     *
     * @return Returns the window handle on success, else -1 on failure.
     */
    WId requestVideoSurface();

    /**
     * Release the video surface from the main window.
     */
    void releaseVideoSurface();

    /**
     * Connect the media player events to the current player controls.
     *
     * @param mediaPlayer The media player to connect to.
     */
    void connectMediaPlayerEvents(MediaPlayer *mediaPlayer);

public slots:
    /**
     * Invoked when the UI needs to be hidden.
     */
    void onHideUI();

    /**
     * Invoked when the media player state has been changed.
     *
     * @param newState The new player state.
     */
    void onStateChanged(MediaPlayerState newState);

private:
    Ui::PopcornPlayerWindow *ui;
    QTimer *_fadeTimer;
    MediaPlayer *_mediaPlayer;
    Log *_log;

    void initializeUi();

    void connectEvents();

    void showOverlay();

    void hideOverlay();

    void togglePlayback();

    /**
     * Update the time of the media player with the given offset.
     *
     * @param offset The offset to apply to the time.
     */
    void updateTime(long offset);

    /**
     * Update the current subtitle offset with the given offset.
     * This will add the offset on top of current subtitle offset.
     *
     * @param offset The offset to add.
     */
    void updateSubtitleOffset(long offset);

protected:
    void keyPressEvent(QKeyEvent *event) override;
    void resizeEvent(QResizeEvent *event) override;
};

#endif // POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
