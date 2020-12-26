#include "SubtitleOffset.h"

#include <QTimer>
#include <iomanip>
#include <sstream>

using namespace std;

SubtitleOffset::SubtitleOffset(QWidget *parent)
    : QLabel(parent)
{
    this->_fadeTimer = new QTimer(this);

    init();
}

void SubtitleOffset::showOffset(long offset)
{
    std::ostringstream streamObj;
    auto offsetInSeconds = (double)offset / (1000 * 1000);

    streamObj << std::setprecision(2);
    streamObj << offsetInSeconds;

    auto prefix = offset >= 0 ? "+" : "";
    auto text = prefix + streamObj.str() + " sec";

    setText(text.c_str());

    _fadeTimer->start();
    this->show();
    repaint();
}

void SubtitleOffset::onHide()
{
    this->hide();
}

void SubtitleOffset::init()
{
    this->_fadeTimer->setSingleShot(true);
    this->_fadeTimer->setInterval(2000);

    connect(this->_fadeTimer, &QTimer::timeout,
        this, &SubtitleOffset::onHide);

    QPalette plt = palette();
    plt.setColor(QPalette::Window, Qt::black);

    this->setPalette(plt);
    this->setAutoFillBackground(true);

    this->setContentsMargins(QMargins(10, 10, 0, 0));
    this->hide();
}
