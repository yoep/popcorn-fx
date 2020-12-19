#include "ProgressControl.h"

#include <QTimer>
#include <QtGui/QPainter>
#include <cmath>

ProgressControl::ProgressControl(QWidget *parent)
    : QFrame(parent)
{
    this->_time = 0;
    this->_duration = 0;
    this->_loadProgress = 0;

    init();
}

void ProgressControl::onTimeChanged(long newValue)
{
    this->_time = newValue;
    emit this->changed();
}

void ProgressControl::onDurationChanged(long newValue)
{
    this->_duration = newValue;
    emit this->changed();
}

void ProgressControl::onDraw()
{
    // check if the progress control is visible
    // if not, ignore the draw event
    if (isVisible())
        repaint();
}

void ProgressControl::init()
{
    connect(this, &ProgressControl::changed,
        this, &ProgressControl::onDraw);
}

void ProgressControl::paintEvent(QPaintEvent *event)
{
    QFrame::paintEvent(event);

    drawTrack();
    drawLoadProgress();
    drawProgress();
}

void ProgressControl::drawTrack()
{
    QPainter painter(this);
    const auto &margins = contentsMargins();
    int width = this->width() - margins.left() - margins.right();
    int height = calculateDrawHeight();

    painter.setBrush(QColor::fromRgb(204, 204, 204));
    painter.drawRect(QRect(margins.left(), margins.top(), width, height));
}

void ProgressControl::drawLoadProgress()
{
    QPainter painter(this);
    const auto &margins = contentsMargins();
    int width = calculateDrawWidth(_loadProgress, 100);
    int height = calculateDrawHeight();

    painter.setBrush(QColor::fromRgb(45, 114, 217));
    painter.drawRect(QRect(margins.left(), margins.top(), width, height));
}

void ProgressControl::drawProgress()
{
    QPainter painter(this);
    const auto &margins = contentsMargins();
    int width = calculateDrawWidth(this->_time, this->_duration);
    int height = calculateDrawHeight();

    painter.setBrush(QColor::fromRgb(45, 114, 217));
    painter.drawRect(QRect(margins.left(), margins.top(), width, height));
}

int ProgressControl::calculateDrawWidth(long value, long max)
{
    const auto &margins = contentsMargins();

    // check if value is larger than max
    // if so, redefine the value as max
    if (value > max)
        value = max;

    int maxWidth = width() - margins.left() - margins.right();
    double width = ((double)maxWidth / max) * value;

    return ceil(width);
}

int ProgressControl::calculateDrawHeight()
{
    const auto &margins = contentsMargins();

    return height() - margins.top() - margins.bottom();
}
