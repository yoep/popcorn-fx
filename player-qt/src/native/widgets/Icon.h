#ifndef POPCORNPLAYER_ICON_H
#define POPCORNPLAYER_ICON_H

#include <QtWidgets/QLabel>

class Icon : public QLabel {
    Q_OBJECT

public:
    Icon(QWidget *parent = nullptr);

signals:
    /**
     * Signals that an action should be triggered for this Icon.
     * This is most of the time invoked when the enter key is pressed.
     */
    void triggerAction();

private:
    void initializeFont();

protected:
    void keyPressEvent(QKeyEvent *ev) override;
};

#endif //POPCORNPLAYER_ICON_H
