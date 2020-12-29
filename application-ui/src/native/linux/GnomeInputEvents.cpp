#include "GnomeInputEvents.h"

#include <gio/gio.h>
#include <iostream>
#include <thread>

GnomeInputEvents::GnomeInputEvents()
{
    this->_log = Log::instance();

    this->_loop = nullptr;
    this->_proxy = nullptr;

    init();
}

GnomeInputEvents::~GnomeInputEvents()
{
    _log->trace("Releasing the Gnome input events resource");

    // check if a proxy was created
    // if so, free all resources used by the proxy
    if (_proxy) {
        releaseProxy();
    }
}

void GnomeInputEvents::init()
{
    _log->trace("Initializing GnomeInputEvents");
    createDBusConnection();
}

void GnomeInputEvents::createDBusConnection()
{
    GError *error = nullptr;
    GDBusProxyFlags flags;

    _loop = g_main_loop_new(nullptr, FALSE);

    // request a new dbus connection
    _log->trace("Trying to establish a new DBus connection");
    _proxy = g_dbus_proxy_new_for_bus_sync(G_BUS_TYPE_SESSION, flags, nullptr,
        "org.gnome.SettingsDaemon.MediaKeys",
        "/org/gnome/SettingsDaemon/MediaKeys",
        "org.gnome.SettingsDaemon.MediaKeys",
        nullptr, &error);

    // check if the connection was created with success
    if (_proxy) {
        _log->debug("Connection to DBus has been established");
        _log->trace("Registering signal callback");
        g_signal_connect(_proxy, "g-signal", G_CALLBACK(onMediaKeyPressed), this);

        grabMediaKeys();

        _gThread = std::thread([&] {
            _log->trace("Starting gmain loop");
            g_main_loop_run(_loop);
        });
    } else {
        handleDBusError(error);
    }
}

void GnomeInputEvents::handleDBusError(GError *error)
{
    auto *message = error->message;

    if (message != nullptr) {
        _log->error(std::string("Failed to create DBus connection, ") + message);
    } else {
        _log->error("Failed to create DBus connection");
    }
}

void GnomeInputEvents::releaseProxy()
{
    releaseMediaKeys();

    g_object_unref(_proxy);
    g_main_loop_quit(_loop);

    // wait for the loop thread to end
    if (_gThread.joinable()) {
        _gThread.join();
    }
}

void GnomeInputEvents::grabMediaKeys()
{
    _log->debug("Grabbing the media player keys");
    g_dbus_proxy_call(_proxy, "GrabMediaPlayerKeys", g_variant_new("(su)", "PopcornKeys", 0),
        G_DBUS_CALL_FLAGS_NO_AUTO_START, -1, nullptr, nullptr, nullptr);
}

void GnomeInputEvents::releaseMediaKeys()
{
    _log->debug("Releasing the media player keys");
    g_dbus_proxy_call(_proxy, "ReleaseMediaPlayerKeys", g_variant_new("(s)", "PopcornKeys"),
        G_DBUS_CALL_FLAGS_NO_AUTO_START, -1, nullptr, nullptr, nullptr);
}

void GnomeInputEvents::onMediaKeyPressed(GDBusProxy *proxy, gchar *sender_name, gchar *signal_name, GVariant *parameters, gpointer instance)
{
    Log *log = Log::instance();

    // check if the instance is valid
    // if not, throw an error as we'll be unable to do anything with the event
    if (instance == nullptr) {
        log->error("Invalid callback event, GnomeInputEvents instance is NULL");
    }

    auto *inputEvents = static_cast<GnomeInputEvents *>(instance);
}
