#ifndef POPCORNTIME_LINUXINPUTEVENTSBRIDGE_H
#define POPCORNTIME_LINUXINPUTEVENTSBRIDGE_H

#include "../../../../shared/Log.h"

#include <IInputEvents.h>
#include <IInputEventsBridge.h>

class LinuxInputEventsBridge : public IInputEventsBridge {
public:
    LinuxInputEventsBridge();

    ~LinuxInputEventsBridge();

private:
    IInputEvents *_inputEvents;
    Log *_log;

    void init();

    void useGnomeInputEvents();

    void useX11InputEvents();

    /**
     * Check if the current desktop environment is using gnome.
     * This method uses the <code>XDG_CURRENT_DESKTOP</code> environment variable to determine the gnome desktop environment.
     *
     * @return Returns true if the desktop environment is using gnome, else false.
     */
    bool isGnomeDesktop();
};

#endif //POPCORNTIME_LINUXINPUTEVENTSBRIDGE_H
