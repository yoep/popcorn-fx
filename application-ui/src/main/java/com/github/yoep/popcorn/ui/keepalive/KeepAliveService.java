package com.github.yoep.popcorn.ui.keepalive;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * Service which will keep the screen and machine alive by sending random inputs to the system.
 * This will prevent the screen from blanking and the machine from going to standby.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class KeepAliveService {
    private final OptionsService optionsService;
    private final PlatformProvider platformProvider;

    @PostConstruct
    void init() {
        // offload the screensaver functionality to a separate thread
        // as this should not block the startup of the application
        new Thread(this::handleScreensaver, "ScreensaverHandle")
                .start();
    }

    void handleScreensaver() {
        if (!isDisabled()) {
            log.trace("Disabling screensaver");
            platformProvider.disableScreensaver();
        } else {
            log.trace("Screensaver will not be disabled as the option is disabled");
        }
    }

    private boolean isDisabled() {
        return optionsService.options().isKeepAliveDisabled();
    }
}
