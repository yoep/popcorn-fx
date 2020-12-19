#ifndef POPCORNPLAYER_PROGRESSCONTROL_H
#define POPCORNPLAYER_PROGRESSCONTROL_H

#include <QtWidgets/QFrame>

class ProgressControl : public QFrame {
    Q_OBJECT
public:
    ProgressControl(QWidget* parent = nullptr);

public slots:
    /**
     * Invoked when the time is changed of the progress control.
     *
     * @param newValue The new time value.
     */
    void onTimeChanged(long newValue);

    /**
     * Invoked when the duration is changed of the progress control.
     *
     * @param newValue The new duration value.
     */
    void onDurationChanged(long newValue);

private slots:
    /**
     * Redraw all tracks.
     */
    void onDraw();

signals:
    /**
     * Signals that the progress control value(s) have been changed.
     */
    void changed();

protected:
    void paintEvent(QPaintEvent *event) override;

private:
    long _time;
    long _duration;
    double _loadProgress;

    void init();

    /**
     * Draw the background track of the progress control.
     */
    void drawTrack();

    /**
     * Draw the load progress track of the progress control.
     */
    void drawLoadProgress();

    /**
     * Draw the playback progress track of the progress control.
     */
    void drawProgress();

    /**
     * Calculate the draw width based on the given value.
     *
     * @param value The value to draw the width of.
     * @param max  The max that the value could possibly be.
     * @return Returns the calculated width to draw for the given value.
     */
    int calculateDrawWidth(long value, long max);

    int calculateDrawHeight();
};

#endif //POPCORNPLAYER_PROGRESSCONTROL_H
