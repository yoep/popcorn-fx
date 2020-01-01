package com.github.yoep.popcorn.settings.models;

import com.github.yoep.popcorn.PopcornTimeApplication;
import lombok.*;

import java.io.File;
import java.util.Objects;

@EqualsAndHashCode(callSuper = true)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class LoggingSettings extends AbstractSettings {
    public static final String DIRECTORY_PROPERTY = "directory";
    public static final String LOG_FILE_PROPERTY = "logfileEnabled";

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

    public void setDirectory(File directory) {
        if (Objects.equals(this.directory, directory))
            return;

        var oldValue = this.directory;
        this.directory = directory;
        changes.firePropertyChange(DIRECTORY_PROPERTY, oldValue, directory);
    }

    public void setLogfileEnabled(boolean logfileEnabled) {
        if (Objects.equals(this.logfileEnabled, logfileEnabled))
            return;

        var oldValue = this.logfileEnabled;
        this.logfileEnabled = logfileEnabled;
        changes.firePropertyChange(LOG_FILE_PROPERTY, oldValue, logfileEnabled);
    }
}
