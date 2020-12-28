#include "GnomeInputEvents.h"

#include <dbus/dbus-glib.h>
#include <dbus/dbus.h>
#include <glib.h>
#include <iostream>
#include <thread>

GnomeInputEvents::GnomeInputEvents()
{
    this->_log = Log::instance();

    this->_dbusConnection = nullptr;
    this->_proxy = nullptr;
    this->_loop = nullptr;

    init();
}

GnomeInputEvents::~GnomeInputEvents()
{
    if (_loop) {
        g_main_loop_quit(_loop);
        free(_loop);
    }

    if (_proxy) {
        dbus_g_proxy_disconnect_signal(_proxy, "MediaPlayerKeyPressed", G_CALLBACK(onMediaKeyPressed), this);
        free(_proxy);
    }
}

void GnomeInputEvents::init()
{
    _log->trace("Initializing GnomeInputEvents");
    createDBusConnection();
}

void GnomeInputEvents::createDBusConnection()
{
    auto *error = (GError *)malloc(sizeof(GError));

    // request a new dbus connection
    _log->trace("Trying to establish a new DBus connection");
    _dbusConnection = ::dbus_g_bus_get(DBusBusType::DBUS_BUS_SESSION, &error);

    // check if the connection was created with success
    if (_dbusConnection) {
        _log->debug("Connection to DBus has been established");

        createDBusProxy();
    } else {
        auto *message = error->message;

        if (message != nullptr) {
            _log->error(std::string("Failed to create DBus connection, ") + error->message);
        } else {
            _log->error("Failed to create DBus connection");
        }
    }

    g_error_free(error);
}

void GnomeInputEvents::createDBusProxy()
{
    // check if a connection was established before creating a proxy
    // if no connection is present, log an error and exit
    if (!_dbusConnection) {
        _log->error("Unable to create proxy, DBus connection has not been established");
        return;
    }

    auto *error = (GError *)malloc(sizeof(GError));

    _log->trace("Creating DBus proxy");
    _proxy = ::dbus_g_proxy_new_for_name(_dbusConnection, "org.gnome.SettingsDaemon", "/org/gnome/SettingsDaemon/MediaKeys",
        "org.gnome.SettingsDaemon.MediaKeys");

    // check if a proxy could be created
    if (_proxy) {
        _log->debug("DBus proxy created with success");

        auto result = dbus_g_proxy_call(_proxy, "GrabMediaPlayerKeys", &error, G_TYPE_STRING, "WebMediaKeys",
            G_TYPE_UINT, 0, G_TYPE_INVALID, G_TYPE_INVALID);

        if (result) {
            dbus_g_proxy_add_signal(_proxy, "MediaPlayerKeyPressed",
                G_TYPE_STRING, G_TYPE_STRING, G_TYPE_INVALID);

            dbus_g_proxy_connect_signal(_proxy, "MediaPlayerKeyPressed", G_CALLBACK(onMediaKeyPressed),
                this, nullptr);

            _loop = g_main_loop_new(nullptr, false);

            std::thread t([&] {
                g_main_loop_run(_loop);
            });
        } else {
            _log->error(std::string("Failed to grab media player keys, ") + error->message);
        }
    } else {
        _log->error("Failed to create DBus proxy");
    }

    g_error_free(error);
}

void GnomeInputEvents::onMediaKeyPressed(DBusGProxy *proxy, const char *value1, const char *value2, gpointer instance)
{
    // check if the instance data is set
    // if not, log an error and exit as we cannot handle the event correctly
    if (!instance) {
        cerr << "Invalid media key pressed callback, instance data is missing" << endl;
        return;
    }

    auto *inputEvents = static_cast<GnomeInputEvents *>(instance);
    inputEvents->_log->debug(std::string("Received key ") + value2);
}
