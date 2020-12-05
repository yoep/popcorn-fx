#include "Icon.h"

#include <QKeyEvent>

Icon::Icon(QWidget *parent)
    : QLabel(parent)
{
    initializeFont();
}

void Icon::initializeFont()
{
    //    auto font = this->font();
    //    font.setFamily(QString("Font Awesome 5 Free"));
}

void Icon::keyPressEvent(QKeyEvent *ev)
{
    if (ev->key() == Qt::Key_Enter || ev->key() == Qt::Key_Return) {
        emit this->triggerAction();
    }

    QLabel::keyPressEvent(ev);
}
