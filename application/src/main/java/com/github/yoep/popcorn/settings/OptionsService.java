package com.github.yoep.popcorn.settings;

import com.github.yoep.popcorn.settings.models.ApplicationOptions;
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
    private static final String BIG_PICTURE_OPTION = "big-picture";
    private static final String KIOSK_OPTION = "kiosk";

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
        boolean bigPictureMode = arguments.containsOption(BIG_PICTURE_OPTION);
        boolean kioskMode = arguments.containsOption(KIOSK_OPTION);

        if (bigPictureMode)
            log.debug("Activating big-picture mode");
        if (kioskMode)
            log.debug("Activating kiosk mode");

        options = ApplicationOptions.builder()
                .bigPictureModeActivated(bigPictureMode)
                .kioskModeActivated(kioskMode)
                .build();
    }
}
