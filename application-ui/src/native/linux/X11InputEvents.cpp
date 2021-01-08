#include "X11InputEvents.h"

#include "X11/Xlib.h"

#include <xkbcommon/xkbcommon-keysyms.h>

X11InputEvents::X11InputEvents()
{
    this->_log = Log::instance();

    this->_display = XOpenDisplay(nullptr);
    this->_window = XDefaultRootWindow(_display);

    init();
}

X11InputEvents::~X11InputEvents()
{
    this->_keepAlive = false;

    // check if the event thread is still joinable
    if (_eventThread.joinable()) {
        // wait for the event thread to quit
        _eventThread.join();
    }

    unregisterKeys();
}

void X11InputEvents::onMediaKeyPressed(std::function<void(MediaKeyType)> mediaKeyPressed)
{
    _mediaKeyPressed = mediaKeyPressed;
}

void X11InputEvents::init()
{
    _log->trace("Initializing X11 input events");

    _log->trace("Creating new event thread");
    this->_eventThread = std::thread([this] {
        _log->trace("Initializing X init threads");
        XInitThreads();

        // register keys
        registerKeys();

        auto event = new XEvent();

        while (_keepAlive) {
            while (XPending(_display) > 0) {
                XNextEvent(_display, event);

                if (event->type == KeyPress) {
                    processEvent(event);
                }
            }
        }
    });
}

void X11InputEvents::processEvent(XEvent *event)
{
    auto keycode = event->xkey.keycode;
    MediaKeyType type = MediaKeyType::UNKNOWN;

    switch (keycode) {
    case XKB_KEY_XF86AudioPlay:
        type = MediaKeyType::PLAY;
        break;
    case XKB_KEY_XF86AudioStop:
        type = MediaKeyType::STOP;
        break;
    case XKB_KEY_XF86AudioPrev:
        type = MediaKeyType::PREVIOUS;
        break;
    case XKB_KEY_XF86AudioNext:
        type = MediaKeyType::NEXT;
        break;
    case XKB_KEY_XF86AudioLowerVolume:
        type = MediaKeyType::VOLUME_LOWER;
        break;
    case XKB_KEY_XF86AudioRaiseVolume:
        type = MediaKeyType::VOLUME_HIGHER;
        break;
    default:
        // no-op
        break;
    }

    if (_mediaKeyPressed != nullptr) {
        _mediaKeyPressed(type);
    }
}

void X11InputEvents::registerKeys()
{
    _log->debug("Registering X11 media input keys");
    auto keys = getKeys();

    for (int i = 0; i < sizeof(keys); ++i) {
        auto key = keys[i];

        try {
            _log->trace("Grabbing X11 key: " + std::to_string(key));
            XGrabKey(_display, XKeysymToKeycode(_display, key), 0, _window, true, GrabModeAsync, GrabModeAsync);
        } catch (std::exception &ex) {
            _log->error(std::string("Failed to grab X11 key, ") + ex.what());
        }
    }
}

void X11InputEvents::unregisterKeys()
{
    _log->debug("Releasing X11 media input keys");
    auto keys = getKeys();

    for (int i = 0; i < sizeof(keys); ++i) {
        auto key = keys[i];

        try {
            _log->trace("Releasing X11 key: " + std::to_string(key));
            XUngrabKey(_display, XKeysymToKeycode(_display, key), 0, _window);
        } catch (std::exception &ex) {
            _log->error(std::string("Failed to release X11 key, ") + ex.what());
        }
    }
}

int *X11InputEvents::getKeys()
{
    static int keys[6];

    keys[0] = XKB_KEY_XF86AudioPlay;
    keys[1] = XKB_KEY_XF86AudioStop;
    keys[2] = XKB_KEY_XF86AudioPrev;
    keys[3] = XKB_KEY_XF86AudioNext;
    keys[4] = XKB_KEY_XF86AudioLowerVolume;
    keys[5] = XKB_KEY_XF86AudioRaiseVolume;

    return keys;
}
