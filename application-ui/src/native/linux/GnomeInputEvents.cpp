#include "GnomeInputEvents.h"

#include <gio/gio.h>
#include <iostream>
#include <thread>

const char *REGISTRATION_NAME = "PopcornKeys";

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

void GnomeInputEvents::onMediaKeyPressed(std::function<void(MediaKeyType)> mediaKeyPressed)
{
    _mediaKeyPressed = mediaKeyPressed;
}

bool GnomeInputEvents::grabMediaKeys()
{
    try {
        _log->debug("Grabbing the media player keys");
        g_dbus_proxy_call(_proxy, "GrabMediaPlayerKeys", g_variant_new("(su)", REGISTRATION_NAME, 0),
            G_DBUS_CALL_FLAGS_NO_AUTO_START, -1, nullptr, &onGrabKeysReady, this);

        return true;
    } catch (std::exception &ex) {
        _log->error("Failed to grab Gnome keys", ex);
        return false;
    }
}

bool GnomeInputEvents::releaseMediaKeys()
{
    try {
        _log->debug("Releasing the media player keys");
        g_dbus_proxy_call(_proxy, "ReleaseMediaPlayerKeys", g_variant_new("(s)", REGISTRATION_NAME),
            G_DBUS_CALL_FLAGS_NO_AUTO_START, -1, nullptr, nullptr, nullptr);

        return true;
    } catch (std::exception &ex) {
        _log->error("Failed to release Gnome keys", ex);
        return false;
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
    GDBusProxyFlags flags = G_DBUS_PROXY_FLAGS_DO_NOT_AUTO_START_AT_CONSTRUCTION;

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
        auto result = g_signal_connect(_proxy, "g-signal", G_CALLBACK(&onGnomeMediaKeyPressed), this);

        // check if the signal could be connected with success
        // if not, log an error
        if (result <= 0) {
            _log->error("Failed to connect DBus signal");
        }

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

void GnomeInputEvents::handleMediaCommand(char *command)
{
    _log->debug(std::string("Received media command ") + command);
    MediaKeyType type = MediaKeyType::UNKNOWN;

    if (command == std::string("Play")) {
        type = MediaKeyType::PLAY;
    } else if (command == std::string("Pause")) {
        type = MediaKeyType::PAUSE;
    } else if (command == std::string("Stop")) {
        type = MediaKeyType::STOP;
    } else if (command == std::string("Previous")) {
        type = MediaKeyType::PREVIOUS;
    } else if (command == std::string("Next")) {
        type = MediaKeyType::NEXT;
    }

    if (_mediaKeyPressed != nullptr) {
        _mediaKeyPressed(type);
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

void GnomeInputEvents::onGrabKeysReady(GObject *source_object, GAsyncResult *res, gpointer instance)
{
    Log *log = Log::instance();

    // check if the instance is valid
    // if not, throw an error as we'll be unable to do anything with the event
    if (instance == nullptr) {
        log->error("Invalid ready callback event, GnomeInputEvents instance is NULL");
    }

    log->debug("Media keys have been grabbed");
}

void GnomeInputEvents::onGnomeMediaKeyPressed(GDBusProxy *proxy, gchar *sender_name, gchar *signal_name, GVariant *parameters, gpointer instance)
{
    Log *log = Log::instance();
    log->trace(std::string("Received signal ") + signal_name);

    // check if the instance is valid
    // if not, throw an error as we'll be unable to do anything with the event
    if (instance == nullptr) {
        log->error("Invalid callback event, GnomeInputEvents instance is NULL");
    }

    auto *inputEvents = static_cast<GnomeInputEvents *>(instance);
    auto type = g_variant_get_type_string(parameters);
    auto children = g_variant_n_children(parameters);

    // check if the incoming parameters matches the expected type
    if (type == std::string("(ss)") && children == 2) {
        log->trace("Received valid media key press event");
        auto applicationName = getParameterValue(parameters, 0);
        auto command = getParameterValue(parameters, 1);

        // verify if the application name is the expected name
        log->trace("Verifying destination application of signal");
        if (applicationName == std::string(REGISTRATION_NAME)) {
            inputEvents->handleMediaCommand(command);
        }

        g_free(applicationName);
        g_free(command);
    } else {
        log->debug(std::string("Unexpected parameters type ") + type + " received");
    }
}

char *GnomeInputEvents::getParameterValue(GVariant *parameters, int index)
{
    auto *rawParameter = g_variant_get_child_value(parameters, index);
    auto parameterSize = g_variant_get_size(rawParameter);
    auto value = g_variant_dup_string(rawParameter, &parameterSize);

    return value;
}
