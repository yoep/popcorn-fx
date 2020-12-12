#ifndef POPCORNPLAYER_BUFFERINDICATOR_H
#define POPCORNPLAYER_BUFFERINDICATOR_H

#include <QtWidgets/QFrame>

class BufferIndicator : public QFrame {
    Q_OBJECT
protected:
    void paintEvent(QPaintEvent *event) override;

public:
    BufferIndicator(QWidget *parent);

    ~BufferIndicator();

private slots:
    void onAnimationTick();

private:
    int _dots = 12;
    int _dotRadius = 20;
    int _animationIndex;
    QTimer *_animationTimer;

    void init();

    QPoint calculateDotPosition(int dotIndex);

    int calculateColorAlpha(int dotIndex);

    int calculateRadius();
};

#endif //POPCORNPLAYER_BUFFERINDICATOR_H
