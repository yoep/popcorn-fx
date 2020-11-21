#ifndef POPCORNPLAYER_PLAYERHEADER_H
#define POPCORNPLAYER_PLAYERHEADER_H

#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE
namespace Ui {
class PlayerHeader;
}
QT_END_NAMESPACE

class PlayerHeader : public QWidget {
    Q_OBJECT

public:
    PlayerHeader(QWidget *parent = nullptr);

    ~PlayerHeader();

private:
    Ui::PlayerHeader *ui;

    void initializeUi();
};

#endif //POPCORNPLAYER_PLAYERHEADER_H
