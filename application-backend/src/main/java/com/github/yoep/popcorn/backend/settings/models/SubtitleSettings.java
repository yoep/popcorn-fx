package com.github.yoep.popcorn.backend.settings.models;

import com.github.yoep.popcorn.backend.settings.SettingsDefaults;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import lombok.*;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class SubtitleSettings extends AbstractSettings {
    public static final String DEFAULT_SUBTITLE_PROPERTY = "defaultSubtitle";
    public static final String DIRECTORY_PROPERTY = "directory";
    public static final String AUTO_CLEANING_PROPERTY = "autoCleaningEnabled";
    public static final String FONT_FAMILY_PROPERTY = "fontFamily";
    public static final String FONT_SIZE_PROPERTY = "fontSize";
    public static final String DECORATION_PROPERTY = "decoration";
    public static final String BOLD_PROPERTY = "bold";

    private static final String DEFAULT_SUBTITLE_DIRECTORY = "subtitles";
    private static final String DEFAULT_FONT_FAMILY = "Arial";
    private static final DecorationType DEFAULT_DECORATION = DecorationType.OUTLINE;
    private static final int DEFAULT_SIZE = 28;

    //region Properties

    /**
     * The directory to save the subtitles to.
     */
    @Builder.Default
    private File directory = new File(SettingsDefaults.APP_DIR + DEFAULT_SUBTITLE_DIRECTORY);
    /**
     * The indication if the subtitle directory should be cleaned if the application is closed.
     */
    @Builder.Default
    private boolean autoCleaningEnabled = true;
    /**
     * The default subtitle language to select for the media playback.
     */
    @Builder.Default
    private SubtitleLanguage defaultSubtitle = SubtitleLanguage.NONE;
    /**
     * The font family to use for the subtitles.
     */
    @Builder.Default
    private SubtitleFamily fontFamily = SubtitleFamily.ARIAL;
    /**
     * The size of the subtitle font.
     */
    @Builder.Default
    private int fontSize = DEFAULT_SIZE;
    /**
     * The subtitle decoration type.
     */
    @Builder.Default
    private DecorationType decoration = DEFAULT_DECORATION;
    /**
     * The indication if the subtitle must always be in the style "bold".
     */
    private boolean bold;

    //endregion

    //region Setters

    public void setDefaultSubtitle(SubtitleLanguage defaultSubtitle) {
        if (Objects.equals(this.defaultSubtitle, defaultSubtitle))
            return;

        var oldValue = this.defaultSubtitle;
        this.defaultSubtitle = defaultSubtitle;
        changes.firePropertyChange(DEFAULT_SUBTITLE_PROPERTY, oldValue, defaultSubtitle);
    }

    public void setDirectory(File directory) {
        if (Objects.equals(this.directory, directory))
            return;

        var oldValue = this.directory;
        this.directory = directory;
        changes.firePropertyChange(DIRECTORY_PROPERTY, oldValue, directory);
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        if (Objects.equals(this.autoCleaningEnabled, autoCleaningEnabled))
            return;

        var oldValue = this.autoCleaningEnabled;
        this.autoCleaningEnabled = autoCleaningEnabled;
        changes.firePropertyChange(AUTO_CLEANING_PROPERTY, oldValue, autoCleaningEnabled);
    }

    public void setFontFamily(SubtitleFamily fontFamily) {
        if (Objects.equals(this.fontFamily, fontFamily))
            return;

        var oldValue = this.fontFamily;
        this.fontFamily = fontFamily;
        changes.firePropertyChange(FONT_FAMILY_PROPERTY, oldValue, fontFamily);
    }

    public void setFontSize(int fontSize) {
        if (Objects.equals(this.fontSize, fontSize))
            return;

        var oldValue = this.fontSize;
        this.fontSize = fontSize;
        changes.firePropertyChange(FONT_SIZE_PROPERTY, oldValue, fontSize);
    }

    public void setDecoration(DecorationType decoration) {
        if (Objects.equals(this.decoration, decoration))
            return;

        var oldValue = this.decoration;
        this.decoration = decoration;
        changes.firePropertyChange(DECORATION_PROPERTY, oldValue, decoration);
    }

    public void setBold(boolean bold) {
        if (Objects.equals(this.bold, bold))
            return;

        var oldValue = this.bold;
        this.bold = bold;
        changes.firePropertyChange(BOLD_PROPERTY, oldValue, bold);
    }

    //endregion

    //region Methods

    public static List<Integer> supportedFontSizes() {
        var sizes = new ArrayList<Integer>();

        // increase sizes always by 2
        for (int i = 20; i <= 80; i += 2) {
            sizes.add(i);
        }

        return sizes;
    }

    //endregion
}
