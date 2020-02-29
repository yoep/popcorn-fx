package com.github.yoep.popcorn.settings;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.settings.models.ApplicationSettings;
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
    private final OptionsService optionsService;

    private ApplicationSettings currentSettings;

    //region Getters

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public ApplicationSettings getSettings() {
        return currentSettings;
    }

    //endregion

    //region Methods

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
    public void save(ApplicationSettings settings) {
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

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        createApplicationSettingsDirectory();
        initializeSettings();
    }

    private void initializeSettings() {
        this.currentSettings = loadSettingsFromFile().orElse(ApplicationSettings.builder().build());
        UISettings uiSettings = this.currentSettings.getUiSettings();

        uiSettings.addListener(event -> {
            if (event.getPropertyName().equals(UISettings.UI_SCALE_PROPERTY)) {
                var uiScale = (UIScale) event.getNewValue();

                updateUIScale(uiScale.getValue());
            }
        });

        updateUIScale(uiSettings.getUiScale().getValue());
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        save();
    }

    //endregion

    //region Functions

    private void updateUIScale(float scale) {
        var options = optionsService.options();

        // check if the big-picture mode is activated
        // if so, double the UI scale
        if (options.isBigPictureModeActivated())
            scale *= 2;

        viewLoader.setScale(scale);
    }

    private Optional<ApplicationSettings> loadSettingsFromFile() {
        File settingsFile = getSettingsFile();

        if (settingsFile.exists()) {
            try {
                log.info("Loading application settings from {}", settingsFile.getAbsolutePath());

                return Optional.of(objectMapper.readValue(settingsFile, ApplicationSettings.class));
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

    //endregion
}
