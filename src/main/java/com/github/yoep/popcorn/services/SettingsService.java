package com.github.yoep.popcorn.services;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.models.settings.Settings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;

@Slf4j
@Service
@RequiredArgsConstructor
public class SettingsService {
    private static final String NAME = "settings.json";
    private final ObjectMapper objectMapper;

    private Settings currentSettings;

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public Settings getSettings() {
        return currentSettings;
    }

    /**
     * Save the current application settings.
     */
    public void save() {
        File settingsFile = getSettingsFile();

        try {
            log.info("Saving application settings to {}", settingsFile.getAbsolutePath());
            FileUtils.writeStringToFile(settingsFile, objectMapper.writeValueAsString(currentSettings), Charset.defaultCharset());
        } catch (IOException ex) {
            throw new SettingsException("Unable to write settings to " + settingsFile.getAbsolutePath(), ex);
        }
    }

    @PostConstruct
    private void init() {
        createApplicationSettingsDirectory();
        loadSettingsFromFile();
    }

    @PreDestroy
    private void destroy() {
        save();
    }

    private void loadSettingsFromFile() {
        File settingsFile = getSettingsFile();

        if (settingsFile.exists()) {
            try {
                log.info("Loading application settings from {}", settingsFile.getAbsolutePath());

                currentSettings = objectMapper.readValue(settingsFile, Settings.class);
            } catch (IOException ex) {
                throw new SettingsException("Unable to read settings file at " + settingsFile.getAbsolutePath(), ex);
            }
        } else {
            currentSettings = Settings.builder().build();
        }
    }

    private void createApplicationSettingsDirectory() {
        File appDir = new File(PopcornTimeApplication.APP_DIR);

        if (!appDir.exists()) {
            if (!appDir.mkdirs()) {
                log.error("Unable to create application directory in " + appDir.getAbsolutePath());
            }
        }
    }

    private File getSettingsFile() {
        return new File(PopcornTimeApplication.APP_DIR + NAME);
    }
}
