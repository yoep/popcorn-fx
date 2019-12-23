package com.github.yoep.popcorn.settings;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.settings.models.Settings;
import com.github.yoep.popcorn.settings.models.UIScale;
import com.github.yoep.popcorn.settings.models.UISettings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class SettingsService {
    private static final String NAME = "settings.json";
    private final ObjectMapper objectMapper;
    private final ViewLoader viewLoader;

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
     * Save the current application settings to the {@link #NAME} file.
     */
    public void save() {
        save(currentSettings);
    }

    /**
     * Save the given application settings.
     * This replaces the currently stored settings.
     *
     * @param settings The application settings to save.
     */
    public void save(Settings settings) {
        File settingsFile = getSettingsFile();

        if (settings != currentSettings)
            currentSettings = settings;

        try {
            log.info("Saving application settings to {}", settingsFile.getAbsolutePath());
            FileUtils.writeStringToFile(settingsFile, objectMapper.writeValueAsString(settings), Charset.defaultCharset());
        } catch (IOException ex) {
            throw new SettingsException("Unable to write settings to " + settingsFile.getAbsolutePath(), ex);
        }
    }

    @PostConstruct
    private void init() {
        createApplicationSettingsDirectory();
        initializeSettings();
    }

    @PreDestroy
    private void destroy() {
        save();
    }

    private void initializeSettings() {
        this.currentSettings = loadSettingsFromFile().orElse(Settings.builder().build());
        UISettings uiSettings = this.currentSettings.getUiSettings();

        uiSettings.addListener(event -> {
            if (event.getPropertyName().equals(UISettings.UI_SCALE_PROPERTY)) {
                UIScale uiScale = (UIScale) event.getNewValue();

                viewLoader.setScale(uiScale.getValue());
            }
        });
        viewLoader.setScale(uiSettings.getUiScale().getValue());
    }

    private Optional<Settings> loadSettingsFromFile() {
        File settingsFile = getSettingsFile();

        if (settingsFile.exists()) {
            try {
                log.info("Loading application settings from {}", settingsFile.getAbsolutePath());

                return Optional.of(objectMapper.readValue(settingsFile, Settings.class));
            } catch (IOException ex) {
                throw new SettingsException("Unable to read settings file at " + settingsFile.getAbsolutePath(), ex);
            }
        }

        log.debug("Using default application settings");
        return Optional.empty();
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
