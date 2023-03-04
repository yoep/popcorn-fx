package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
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
    public static final String DISABLE_MOUSE_OPTION = "disable-mouse";

    private final ApplicationArguments arguments;
    private final FxLib fxLib;
    private final PopcornFx instance;

    private ApplicationOptions options;

    /**
     * Get the application options of the application.
     *
     * @return Returns the options for this run instance of the application.
     */
    public ApplicationOptions options() {
        return options;
    }

    public boolean isTvMode() {
        return fxLib.is_tv_mode(instance) == 1;
    }

    public boolean isMaximized() {
        return fxLib.is_maximized(instance) == 1;
    }

    @PostConstruct
    void init() {
        var bigPictureMode = arguments.containsOption(BIG_PICTURE_MODE_OPTION);
        var kioskMode = arguments.containsOption(KIOSK_MODE_OPTION);
        var disableMouse = arguments.containsOption(DISABLE_MOUSE_OPTION);

        if (bigPictureMode)
            log.debug("Activating big-picture mode");
        if (kioskMode)
            log.debug("Activating kiosk mode");
        if (disableMouse)
            log.debug("Disabling mouse application wide");

        options = ApplicationOptions.builder()
                .bigPictureMode(bigPictureMode)
                .kioskMode(kioskMode)
                .mouseDisabled(disableMouse)
                .build();
    }
}
