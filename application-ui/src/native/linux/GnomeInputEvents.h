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

    void onMediaKeyPressed(std::function<void(MediaKeyType)> mediaKeyPressed) override;

private:
    GMainLoop *_loop;
    GDBusProxy *_proxy;
    std::thread _gThread;
    std::function<void(MediaKeyType)> _mediaKeyPressed;
    Log *_log;

    void init();

    void createDBusConnection();

    /**
     * Handle the given DBus error.
     *
     * @param error The error to process.
     */
    void handleDBusError(GError *error);

    /**
     * Handle the given received Gnome command.
     *
     * @param command The command to process.
     */
    void handleMediaCommand(char *command);

    /**
     * Release the proxy resources.
     */
    void releaseProxy();

    /**
     * Grab the media player keys from Gnome.
     */
    void grabMediaKeys();

    /**
     * Release the grabbed media keys back to Gnome.
     */
    void releaseMediaKeys();

    static void onGrabKeysReady(GObject *source_object, GAsyncResult *res, gpointer instance);

    /**
     * Invoked when a media key has been pressed.
     *
     * @param proxy The proxy which invoked the callback.
     * @param sender_name The sender of the callback.
     * @param signal_name The signal name.
     * @param parameters
     * @param instance The GnomeInputEvents instance data.
     */
    static void onGnomeMediaKeyPressed(GDBusProxy *proxy, gchar *sender_name, gchar *signal_name, GVariant *parameters, gpointer instance);

    /**
     * Get the parameter string value from the parameters at the given index.
     *
     * @param parameters The parameters.
     * @param index The parameter index to retrieve the value of.
     * @return Returns the char array value of the parameter.
     */
    static char *getParameterValue(GVariant *parameters, int index);
};

#endif //POPCORNTIME_GNOMEINPUTEVENTS_H
