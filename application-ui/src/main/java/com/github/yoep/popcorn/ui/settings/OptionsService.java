package com.github.yoep.popcorn.ui.settings;

import com.github.yoep.popcorn.ui.settings.models.ApplicationOptions;
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
    private void init() {
        var bigPictureMode = arguments.containsOption(BIG_PICTURE_MODE_OPTION);
        var kioskMode = arguments.containsOption(KIOSK_MODE_OPTION);
        var tvMode = arguments.containsOption(TV_MODE_OPTION);

        if (bigPictureMode)
            log.debug("Activating big-picture mode");
        if (kioskMode)
            log.debug("Activating kiosk mode");
        if (tvMode)
            log.debug("Activating tv mode");

        options = ApplicationOptions.builder()
                .bigPictureMode(bigPictureMode)
                .kioskMode(kioskMode)
                .tvMode(tvMode)
                .build();
    }
}
