#ifndef POPCORNPLAYER_TIMELABEL_H
#define POPCORNPLAYER_TIMELABEL_H

#include <QLabel>

class TimeLabel : public QLabel {
public:
    /**
     * Initialize a new instance of TimeLabel.
     *
     * @param parent The parent of this time label.
     */
    TimeLabel(QWidget *parent = nullptr);

    /**
     * Set the time of this TimeLabel.
     *
     * @param millis The milliseconds value of this time label.
     */
    void setTime(long millis);

    /**
     * Reset the time label to it's default placeholder text.
     */
    void reset();

private:
    static QString toDisplayTime(long millis);
};

#endif //POPCORNPLAYER_TIMELABEL_H
