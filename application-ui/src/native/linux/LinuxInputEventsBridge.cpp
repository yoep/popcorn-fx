#include "LinuxInputEventsBridge.h"

#include "GnomeInputEvents.h"
#include "X11InputEvents.h"

#include <regex>

using namespace std;

LinuxInputEventsBridge::LinuxInputEventsBridge()
    : IInputEventsBridge()
{
    this->_log = Log::instance();

    this->_inputEvents = nullptr;

    init();
}

LinuxInputEventsBridge::~LinuxInputEventsBridge()
{
    _log->trace("Releasing the linux input events bridge resources");
    delete _inputEvents;
}

void LinuxInputEventsBridge::init()
{
    _log->debug("Using linux inputs event bridge");
    bool gnomeDetected = isGnomeDesktop();

    if (gnomeDetected) {
        useGnomeInputEvents();
    } else {
        useX11InputEvents();
    }

    _log->debug("Linux inputs event bridge initialized");
}

void LinuxInputEventsBridge::useGnomeInputEvents()
{
    _log->info("Using Gnome key input events");
    _inputEvents = new GnomeInputEvents();
}

void LinuxInputEventsBridge::useX11InputEvents()
{
    _log->info("Using X11 key input events");
    _inputEvents = new X11InputEvents();
}

bool LinuxInputEventsBridge::isGnomeDesktop()
{
    char *desktop = getenv("XDG_CURRENT_DESKTOP");
    auto gnomeRegex = std::regex(".*(:gnome)", std::regex_constants::icase);

    if (strlen(desktop) > 0) {
        _log->trace(std::string("Detected desktop type: \"") + desktop + "\"");
        return regex_search(desktop, gnomeRegex);
    } else {
        _log->warn("Unable to detect desktop type, falling back to X11");
        return false;
    }
}
