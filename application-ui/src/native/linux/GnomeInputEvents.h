#ifndef POPCORNTIME_GNOMEINPUTEVENTS_H
#define POPCORNTIME_GNOMEINPUTEVENTS_H

#include "../../../../shared/Log.h"

#include <IInputEvents.h>
#include <dbus/dbus-glib.h>

class GnomeInputEvents : public IInputEvents {
public:
    GnomeInputEvents();

    virtual ~GnomeInputEvents();

private:
    DBusGConnection *_dbusConnection;
    DBusGProxy *_proxy;
    GMainLoop *_loop;
    Log *_log;

    void init();

    void createDBusConnection();

    void createDBusProxy();

    /**
     * Invoked when one of the media keys is pressed within Gnome.
     *
     * @param proxy The proxy which invoked the method.
     * @param value1
     * @param value2 The media key value.
     * @param user_data
     */
    static void onMediaKeyPressed(DBusGProxy *proxy, const char *value1, const char *value2, gpointer user_data);
};

#endif //POPCORNTIME_GNOMEINPUTEVENTS_H
