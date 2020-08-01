package com.github.yoep.popcorn.ui.torrent.utils;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class SizeUtilsTest {
    @Test
    public void testToDisplaySize_whenSizeIsBytes_ShouldReturnBytes() {
        var value = 10;
        var expectedValue = "10 bytes";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }

    @Test
    public void testToDisplaySize_whenSizeIsKB_ShouldReturnKB() {
        var value = 2048;
        var expectedValue = "2.00 KB";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }

    @Test
    public void testToDisplaySize_whenSizeIsMB_shouldReturnMB() {
        var value = 3145728;
        var expectedValue = "3.00 MB";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }

    @Test
    public void testToDisplaySize_whenSizeIsGB_shouldReturnGB() {
        var value = 4294967296L;
        var expectedValue = "4.00 GB";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }

    @Test
    public void testToDisplaySize_whenSizeIsTB_shouldReturnTB() {
        var value = 5497558138880L;
        var expectedValue = "5.00 TB";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }

    @Test
    public void testToDisplaySize_whenSizeIsNotRound_shouldReturnTheCorrectDecimals() {
        var value = 11010048L;
        var expectedValue = "10.50 MB";

        var result = SizeUtils.toDisplaySize(value);

        assertEquals(expectedValue, result);
    }


}
