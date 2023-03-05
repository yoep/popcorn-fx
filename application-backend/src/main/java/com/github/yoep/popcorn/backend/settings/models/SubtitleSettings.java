package com.github.yoep.popcorn.backend.settings.models;

import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"directory", "autoCleaningEnabled", "defaultSubtitle", "fontFamily", "fontSize", "decoration", "bold"})
public class SubtitleSettings extends Structure implements Closeable {
    public static class ByValue extends SubtitleSettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(SubtitleSettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.directory = settings.directory;
            this.autoCleaningEnabled = settings.autoCleaningEnabled;
            this.defaultSubtitle = settings.defaultSubtitle;
            this.fontFamily = settings.fontFamily;
            this.fontSize = settings.fontSize;
            this.decoration = settings.decoration;
            this.bold = settings.bold;
        }
    }

    //region Properties

    /**
     * The directory to save the subtitles to.
     */
    public String directory;
    /**
     * The indication if the subtitle directory should be cleaned if the application is closed.
     */
    public byte autoCleaningEnabled;
    /**
     * The default subtitle language to select for the media playback.
     */
    public SubtitleLanguage defaultSubtitle;
    /**
     * The font family to use for the subtitles.
     */
    public SubtitleFamily fontFamily;
    /**
     * The size of the subtitle font.
     */
    public int fontSize;
    /**
     * The subtitle decoration type.
     */
    public DecorationType decoration;
    /**
     * The indication if the subtitle must always be in the style "bold".
     */
    public byte bold;

    //endregion

    //region Methods

    public boolean isAutoCleaningEnabled() {
        return autoCleaningEnabled == 1;
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        this.autoCleaningEnabled = (byte) (autoCleaningEnabled ? 1 : 0);
    }

    public boolean isBold() {
        return bold == 1;
    }

    public void setBold(boolean bold) {
        this.bold = (byte) (bold ? 1 : 0);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

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
