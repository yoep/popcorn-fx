package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.settings.models.ApplicationOptions;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * The application options service processes the application argument options.
 * This service builds an model of the options that have been passed to the application.
 * Use {@link #options()} to retrieve the processed options.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class OptionsService {
    public static final String BIG_PICTURE_MODE_OPTION = "big-picture";
    public static final String KIOSK_MODE_OPTION = "kiosk";
    public static final String TV_MODE_OPTION = "tv";
    public static final String MAXIMIZED_OPTION = "maximized";
    public static final String DISABLE_MOUSE_OPTION = "disable-mouse";
    public static final String DISABLE_KEEP_ALIVE_OPTION = "disable-keep-alive";

    private final ApplicationArguments arguments;

    private ApplicationOptions options;

    /**
     * Get the application options of the application.
     *
     * @return Returns the options for this run instance of the application.
     */
    public ApplicationOptions options() {
        return options;
    }

    @PostConstruct
    void init() {
        var bigPictureMode = arguments.containsOption(BIG_PICTURE_MODE_OPTION);
        var kioskMode = arguments.containsOption(KIOSK_MODE_OPTION);
        var tvMode = arguments.containsOption(TV_MODE_OPTION);
        var maximized = arguments.containsOption(MAXIMIZED_OPTION);
        var disableMouse = arguments.containsOption(DISABLE_MOUSE_OPTION);
        var disableKeepAlive = arguments.containsOption(DISABLE_KEEP_ALIVE_OPTION);

        if (bigPictureMode)
            log.debug("Activating big-picture mode");
        if (kioskMode)
            log.debug("Activating kiosk mode");
        if (tvMode)
            log.debug("Activating tv mode");
        if (disableMouse)
            log.debug("Disabling mouse application wide");
        if (disableKeepAlive)
            log.debug("Disabling keep alive service");

        options = ApplicationOptions.builder()
                .bigPictureMode(bigPictureMode)
                .kioskMode(kioskMode)
                .tvMode(tvMode)
                .maximized(maximized)
                .mouseDisabled(disableMouse)
                .keepAliveDisabled(disableKeepAlive)
                .build();
    }
}
