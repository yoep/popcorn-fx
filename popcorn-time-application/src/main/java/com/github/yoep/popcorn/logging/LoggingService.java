package com.github.yoep.popcorn.logging;

import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.LoggingSettings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.core.Appender;
import org.apache.logging.log4j.core.Logger;
import org.apache.logging.log4j.core.appender.FileAppender;
import org.apache.logging.log4j.core.config.Property;
import org.apache.logging.log4j.core.layout.PatternLayout;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.io.File;
import java.time.LocalDate;

@Slf4j
@Service
@RequiredArgsConstructor
public class LoggingService {
    private static final String LOG_APPENDER_NAME = "fileLogger";
    private static final String FILENAME_FORMAT = "popcorn-time-%s.log";

    private final SettingsService settingsService;
    private final Logger coreLogger = (Logger) LogManager.getRootLogger();

    private Appender appender;

    //region Methods

    /**
     * Enable the logfile in the logger (if not already enabled).
     */
    public void enableLogfile() {
        if (!coreLogger.getAppenders().containsKey(LOG_APPENDER_NAME)) {
            log.debug("Adding log file appender \"{}\" to root logger", LOG_APPENDER_NAME);
            coreLogger.addAppender(getLogFileAppender());
        }
    }

    /**
     * Disabled logfile in the logger (if not already disabled).
     */
    public void disableLogfile() {
        if (!coreLogger.getAppenders().containsKey(LOG_APPENDER_NAME)) {
            log.debug("Removing log file appender \"{}\" from root logger", LOG_APPENDER_NAME);
            Appender appender = coreLogger.getAppenders().get(LOG_APPENDER_NAME);
            coreLogger.removeAppender(appender);
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        updateLogger();
    }

    private void updateLogger() {
        LoggingSettings logging = getLoggingSettings();

        if (logging.isLogfileEnabled()) {
            enableLogfile();
        } else {
            disableLogfile();
        }
    }
    //endregion

    //region Functions

    private Appender getLogFileAppender() {
        if (appender == null)
            appender = createFileAppender();

        return appender;
    }

    private Appender createFileAppender() {
        LoggingSettings loggingSettings = getLoggingSettings();

        String filename = String.format(FILENAME_FORMAT, LocalDate.now());
        FileAppender fileAppender = FileAppender.newBuilder()
                .setName(LOG_APPENDER_NAME)
                .withFileName(loggingSettings.getDirectory().getAbsolutePath() + File.separator + filename)
                .withAppend(true)
                .setPropertyArray(new Property[]{
                        Property.createProperty("PID", "????"),
                })
                .setLayout(PatternLayout.newBuilder()
                        .withPattern("%d{yyyy-MM-dd HH:mm:ss.SSS} %5p ${sys:PID} --- [%t] %-40.40c{1.} : %m%n%xwEx")
                        .build())
                .build();

        fileAppender.start();
        return fileAppender;
    }

    private LoggingSettings getLoggingSettings() {
        return settingsService.getSettings().getLoggingSettings();
    }

    //endregion
}
