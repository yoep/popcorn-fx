#ifndef POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
#define POPCORN_PLAYER_POPCORNPLAYERWINDOW_H

#include "PlayerControls.h"
#include "PlayerHeader.h"
#include "widgets/VideoWidget.h"

#include <QMainWindow>
#include <player/Media.h>
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
     * Connect the media events to the current player controls.
     *
     * @param media The media to connect to.
     */
    void connectMediaEvents(Media *media);

    /**
     * Connect the media player events to the current player controls.
     *
     * @param mediaPlayer The media player to connect to.
     */
    void connectMediaPlayerEvents(MediaPlayer *mediaPlayer);

private:
    Ui::PopcornPlayerWindow *ui;

    void initializeUi();

protected:
    void paintEvent(QPaintEvent *event) override;
};

#endif // POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
