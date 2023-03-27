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

    public boolean isKioskMode() {
        return fxLib.is_kiosk_mode(instance) == 1;
    }

    @PostConstruct
    void init() {
        var disableMouse = arguments.containsOption(DISABLE_MOUSE_OPTION);

        if (disableMouse)
            log.debug("Disabling mouse application wide");

        options = ApplicationOptions.builder()
                .mouseDisabled(disableMouse)
                .build();
    }
}
