#include "Log.h"
#include "vlc/vlc.h"

#include <QtWidgets/QFrame>
#include <QtWidgets/QWidget>

#ifndef POPCORN_PLAYER_PLAYER_H
#define POPCORN_PLAYER_PLAYER_H

class VideoWidget : public QFrame {
    Q_OBJECT

public:
    explicit VideoWidget(QWidget *parent = nullptr);

    ~VideoWidget() override;

    /**
     * Request the window handle of this widget.
     * Only one video surface can be active at a time, so make sure to #release the previous one.
     *
     * @return Returns the window handle on success, or -1 on failure.
     */
    WId request();

    /**
     * Release the current video playback surface.
     */
    void release();

private:
    Log *_log;
    QWidget *_videoSurface;
    QLayout *_layout;

    void initializeUi();

    void sync();
};

#endif //POPCORN_PLAYER_PLAYER_H
