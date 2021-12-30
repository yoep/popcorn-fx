package com.github.yoep.popcorn.backend.settings.models;

import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import lombok.*;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

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
    public static final String DEFAULT_SUBTITLE_DIRECTORY = "subtitles";

    private static final String DEFAULT_FONT_FAMILY = "Arial";
    private static final DecorationType DEFAULT_DECORATION = DecorationType.OUTLINE;
    private static final int DEFAULT_SIZE = 28;

    //region Properties

    /**
     * The directory to save the subtitles to.
     */
    private File directory;
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
        this.defaultSubtitle = updateProperty(this.defaultSubtitle, defaultSubtitle, DEFAULT_SUBTITLE_PROPERTY);
    }

    public void setDirectory(File directory) {
        this.directory = updateProperty(this.directory, directory, DIRECTORY_PROPERTY);
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        this.autoCleaningEnabled = updateProperty(this.autoCleaningEnabled, autoCleaningEnabled, AUTO_CLEANING_PROPERTY);
    }

    public void setFontFamily(SubtitleFamily fontFamily) {
        this.fontFamily = updateProperty(this.fontFamily, fontFamily, FONT_FAMILY_PROPERTY);
    }

    public void setFontSize(int fontSize) {
        this.fontSize = updateProperty(this.fontSize, fontSize, FONT_SIZE_PROPERTY);
    }

    public void setDecoration(DecorationType decoration) {
        this.decoration = updateProperty(this.decoration, decoration, DECORATION_PROPERTY);
    }

    public void setBold(boolean bold) {
        this.bold = updateProperty(this.bold, bold, BOLD_PROPERTY);
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
