#include "BufferIndicator.h"

#include <QTimer>
#include <QtGui/QPainter>
#include <cmath>

BufferIndicator::BufferIndicator(QWidget *parent)
    : QFrame(parent)
{
    this->_animationTimer = new QTimer(this);
    this->_animationIndex = 0;

    init();
}

BufferIndicator::~BufferIndicator()
{
    delete _animationTimer;
}

void BufferIndicator::paintEvent(QPaintEvent *event)
{
    QFrame::paintEvent(event);
    QPainter painter(this);

    for (int i = 0; i < _dots; ++i) {
        const QPoint &point = calculateDotPosition(i);

        painter.setPen(QColor::fromRgb(0, 0, 0, 0));
        painter.setBrush(QColor::fromRgb(45, 114, 217, calculateColorAlpha(i)));
        painter.drawEllipse(QRect(point.x(), point.y(), _dotRadius, _dotRadius));
    }
}

void BufferIndicator::init()
{
    _animationTimer->setSingleShot(false);
    _animationTimer->setInterval(200);
    _animationTimer->start();

    connect(_animationTimer, &QTimer::timeout,
        this, &BufferIndicator::onAnimationTick);

    setStyleSheet(QString("background-color: rgb(23, 24, 27);"));
}

QPoint BufferIndicator::calculateDotPosition(int dotIndex)
{
    double slice = 2 * M_PI / _dots;
    double angle = slice * dotIndex;
    int radius = calculateRadius();
    int x = ((this->width() / 2) + radius * cos(angle));
    int y = ((this->height() / 2) + radius * sin(angle));

    return QPoint(x, y);
}

int BufferIndicator::calculateColorAlpha(int dotIndex)
{
    int alphaMax = 255;
    int alphaLeap = alphaMax / (_dots * 0.25);
    int leapStep = _animationIndex - dotIndex;
    int absoluteLeapStep = leapStep < 0 ? _dots : leapStep;
    int alpha = alphaMax - (absoluteLeapStep * alphaLeap);

    if (alpha < 0) {
        return 0;
    } else if (alpha > alphaMax) {
        return alphaMax;
    } else {
        return alpha;
    }
}

int BufferIndicator::calculateRadius()
{
    const QMargins &margins = contentsMargins();
    int originWidth = this->width();
    int originHeight = this->height();
    int width = originWidth - _dotRadius - margins.left() - margins.right();
    int height = originHeight - _dotRadius - margins.top() - margins.bottom();

    if (width < height) {
        return width / 2;
    } else {
        return height / 2;
    }
}

void BufferIndicator::onAnimationTick()
{
    _animationIndex++;

    if (_animationIndex > _dots) {
        _animationIndex = 0;
    }

    if (isVisible()) {
        repaint();
    }
}
