#ifndef POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
#define POPCORN_PLAYER_POPCORNPLAYERWINDOW_H

#include "PlayerControls.h"
#include "PlayerHeader.h"
#include "widgets/VideoWidget.h"

#include <QMainWindow>

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

private:
    VideoWidget *player;
    PlayerHeader *header;
    PlayerControls *controls;

    void initializeUi();

protected:
    void resizeEvent(QResizeEvent *event) override;
};

#endif // POPCORN_PLAYER_POPCORNPLAYERWINDOW_H
