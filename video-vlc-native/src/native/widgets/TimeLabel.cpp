#include "TimeLabel.h"

TimeLabel::TimeLabel(QWidget *parent)
    : QLabel(parent)
{
    reset();
}

void TimeLabel::setTime(long millis)
{
    setText(toDisplayTime(millis));
}

void TimeLabel::reset()
{
    setText("--:--");
}

QString TimeLabel::toDisplayTime(long millis)
{
    int minutes = (int)((millis / (1000 * 60)) % 60);
    int seconds = (int)(millis / 1000) % 60;

    return QString("%1:%2")
        .arg(QString::number(minutes), 2, QLatin1Char('0'))
        .arg(QString::number(seconds), 2, QLatin1Char('0'));
}
