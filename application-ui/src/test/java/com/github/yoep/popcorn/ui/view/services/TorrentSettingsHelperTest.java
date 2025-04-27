package com.github.yoep.popcorn.ui.view.services;

import org.junit.jupiter.api.Test;

import java.text.DecimalFormat;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class TorrentSettingsHelperTest {

    @Test
    void testToDisplayValue_whenValueIs1000Bytes_shouldReturn1KiloByte() {
        var value = 1000;
        var expectedResult = "1";

        var result = TorrentSettingsHelper.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToDisplayValue_whenValueIs3500Bytes_shouldReturn3Dot5KiloByte() {
        var value = 3500;
        var decimalFormat = new DecimalFormat();
        var expectedResult = "3" + decimalFormat.getDecimalFormatSymbols().getDecimalSeparator() + "5";

        var result = TorrentSettingsHelper.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToDisplayValue_whenValueIsZero_shouldReturn0KiloByte() {
        var value = 0;
        var expectedResult = "0";

        var result = TorrentSettingsHelper.toDisplayValue(value);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(NullPointerException.class, () -> TorrentSettingsHelper.toSettingsValue(null), "displayValue cannot be null");
    }

    @Test
    void testToSettingsValue_whenDisplayValueIsEmpty_shouldReturnZero() {
        var displayValue = "";
        var expectedResult = 0;

        var result = TorrentSettingsHelper.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIs1KiloByte_shouldReturn1000Bytes() {
        var displayValue = "1";
        var expectedResult = 1000;

        var result = TorrentSettingsHelper.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToSettingsValue_whenDisplayValueIs3Dot5KiloByte_shouldReturn3500Bytes() {
        var displayValue = "3.5";
        var expectedResult = 3500;

        var result = TorrentSettingsHelper.toSettingsValue(displayValue);

        assertEquals(expectedResult, result);
    }
}
