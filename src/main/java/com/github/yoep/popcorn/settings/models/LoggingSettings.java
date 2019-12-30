package com.github.yoep.popcorn.settings.models;

import com.github.yoep.popcorn.PopcornTimeApplication;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.io.File;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class LoggingSettings {
    private static final String DEFAULT_LOG_DIRECTORY = "logs";

    /**
     * The directory to save the logs to.
     */
    @Builder.Default
    private File directory = new File(PopcornTimeApplication.APP_DIR + DEFAULT_LOG_DIRECTORY);
    /**
     * The indication if logging to a file is enabled for the application.
     */
    @Builder.Default
    private boolean logfileEnabled = true;
}
