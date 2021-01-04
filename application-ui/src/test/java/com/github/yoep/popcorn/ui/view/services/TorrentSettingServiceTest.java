package com.github.yoep.popcorn.ui.view.services;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.text.DecimalFormat;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class TorrentSettingServiceTest {
    private TorrentSettingService torrentSettingService;

    @BeforeEach
    void setUp() {
        torrentSettingService = new TorrentSettingService();
    }

    @Test
    void testToDisplayValue_whenValueIs1000Bytes_shouldReturn1KiloByte() {
        var value = 1000;
        var expectedResult = "1";

        var result = torrentSettingService.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToDisplayValue_whenValueIs3500Bytes_shouldReturn3Dot5KiloByte() {
        var value = 3500;
        var decimalFormat = new DecimalFormat();
        var expectedResult = "3" + decimalFormat.getDecimalFormatSymbols().getDecimalSeparator() + "5";

        var result = torrentSettingService.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToDisplayValue_whenValueIsZero_shouldReturn0KiloByte() {
        var value = 0;
        var expectedResult = "0";

        var result = torrentSettingService.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> torrentSettingService.toSettingsValue(null), "displayValue cannot be null");
    }

    @Test
    void testToSettingsValue_whenDisplayValueIsEmpty_shouldReturnZero() {
        var displayValue = "";
        var expectedResult = 0;

        var result = torrentSettingService.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIs1KiloByte_shouldReturn1000Bytes() {
        var displayValue = "1";
        var expectedResult = 1000;

        var result = torrentSettingService.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIs3Dot5KiloByte_shouldReturn3500Bytes() {
        var displayValue = "3.5";
        var expectedResult = 3500;

        var result = torrentSettingService.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }
}
