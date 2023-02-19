package com.github.yoep.popcorn.backend.settings.models;

import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleFamily;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import org.junit.jupiter.api.Test;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;

class SubtitleSettingsTest extends AbstractPropertyTest<SubtitleSettings> {
    public SubtitleSettingsTest() {
        super(SubtitleSettings.class);
    }

    @Test
    void testSetDefaultSubtitle_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = SubtitleLanguage.ENGLISH;

        settings.setDefaultSubtitle(newValue);

        assertEquals(SubtitleSettings.DEFAULT_SUBTITLE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetDirectory_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = new File("");

        settings.setDirectory(newValue);

        assertEquals(SubtitleSettings.DIRECTORY_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetAutoCleaningEnabled_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = false;

        settings.setAutoCleaningEnabled(newValue);

        assertEquals(SubtitleSettings.AUTO_CLEANING_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetFontFamily_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = SubtitleFamily.TAHOMA;

        settings.setFontFamily(newValue);

        assertEquals(SubtitleSettings.FONT_FAMILY_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetFontSize_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = 13;

        settings.setFontSize(newValue);

        assertEquals(SubtitleSettings.FONT_SIZE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetDecoration_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = DecorationType.OPAQUE_BACKGROUND;

        settings.setDecoration(newValue);

        assertEquals(SubtitleSettings.DECORATION_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetBold_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = true;

        settings.setBold(newValue);

        assertEquals(SubtitleSettings.BOLD_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }
}