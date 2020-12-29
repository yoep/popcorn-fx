#ifndef POPCORNTIME_GNOMEINPUTEVENTS_H
#define POPCORNTIME_GNOMEINPUTEVENTS_H

#include "../../../../shared/Log.h"

#include <IInputEvents.h>
#include <gio/gio.h>
#include <thread>

class GnomeInputEvents : public IInputEvents {
public:
    GnomeInputEvents();

    ~GnomeInputEvents();

private:
    GMainLoop *_loop;
    GDBusProxy *_proxy;
    std::thread _gThread;
    Log *_log;

    void init();

    void createDBusConnection();

    void handleDBusError(GError *error);

    void releaseProxy();

    /**
     * Grab the media player keys from Gnome.
     */
    void grabMediaKeys();

    /**
     * Release the grabbed media keys back to Gnome.
     */
    void releaseMediaKeys();

    /**
     * Invoked when a media key has been pressed.
     *
     * @param proxy The proxy which invoked the callback.
     * @param sender_name The sender of the callback.
     * @param signal_name The signal name.
     * @param parameters
     * @param instance The GnomeInputEvents instance data.
     */
    static void onMediaKeyPressed(GDBusProxy *proxy, gchar *sender_name, gchar *signal_name, GVariant *parameters, gpointer instance);
};

#endif //POPCORNTIME_GNOMEINPUTEVENTS_H
