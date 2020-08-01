package com.github.yoep.popcorn.ui.torrent.utils;

import org.apache.commons.io.FileUtils;

import java.math.BigDecimal;
import java.math.RoundingMode;

public class SizeUtils {
    private static final BigDecimal ONE_TB = BigDecimal.valueOf(FileUtils.ONE_TB_BI.longValue());
    private static final BigDecimal ONE_GB = BigDecimal.valueOf(FileUtils.ONE_GB_BI.longValue());
    private static final BigDecimal ONE_MB = BigDecimal.valueOf(FileUtils.ONE_MB_BI.longValue());
    private static final BigDecimal ONE_KB = BigDecimal.valueOf(FileUtils.ONE_KB_BI.longValue());

    private SizeUtils() {
    }

    /**
     * Convert the given value to a human readable size.
     *
     * @param value The value in bytes to convert.
     * @return Returns the human readable size.
     */
    public static String toDisplaySize(int value) {
        return toDisplaySize((long) value);
    }

    /**
     * Convert the given value to a human readable size.
     *
     * @param value The value in bytes to convert.
     * @return Returns the human readable size.
     */
    public static String toDisplaySize(long value) {
        BigDecimal size = BigDecimal.valueOf(value);

        if (size.divide(ONE_TB).intValue() > 0) {
            return size.divide(ONE_TB).setScale(2, RoundingMode.HALF_UP) + " TB";
        } else if (size.divide(ONE_GB).intValue() > 0) {
            return size.divide(ONE_GB).setScale(2, RoundingMode.HALF_UP) + " GB";
        } else if (size.divide(ONE_MB).intValue() > 0) {
            return size.divide(ONE_MB).setScale(2, RoundingMode.HALF_UP) + " MB";
        } else if (size.divide(ONE_KB).intValue() > 0) {
            return size.divide(ONE_KB).setScale(2, RoundingMode.HALF_UP) + " KB";
        }

        return size.intValue() + " bytes";
    }
}
