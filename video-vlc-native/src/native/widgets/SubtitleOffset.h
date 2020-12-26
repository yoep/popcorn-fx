#ifndef POPCORNPLAYER_SUBTITLEOFFSET_H
#define POPCORNPLAYER_SUBTITLEOFFSET_H

#include <QtWidgets/QLabel>

class SubtitleOffset : public QLabel {
    Q_OBJECT

public:
    SubtitleOffset(QWidget *parent = nullptr);

    /**
     * Show the given offset time.
     *
     * @param offset The offset in microseconds.
     */
    void showOffset(long offset);

private slots:
    void onHide();

private:
    QTimer *_fadeTimer;

    void init();
};

#endif //POPCORNPLAYER_SUBTITLEOFFSET_H
