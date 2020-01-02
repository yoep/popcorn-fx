package com.github.yoep.popcorn.torrent.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import org.apache.commons.io.FileUtils;

import java.math.BigDecimal;
import java.math.RoundingMode;

@Getter
@Builder
@AllArgsConstructor
public class StreamStatus {
    private static final BigDecimal ONE_TB = BigDecimal.valueOf(FileUtils.ONE_TB_BI.longValue());
    private static final BigDecimal ONE_GB = BigDecimal.valueOf(FileUtils.ONE_GB_BI.longValue());
    private static final BigDecimal ONE_MB = BigDecimal.valueOf(FileUtils.ONE_MB_BI.longValue());
    private static final BigDecimal ONE_KB = BigDecimal.valueOf(FileUtils.ONE_KB_BI.longValue());

    /**
     * A value in the range [0, 1], that represents the progress of the torrent's
     * current task. It may be checking files or downloading.
     */
    private final float progress;
    /**
     * The number of peers that are seeding that this client is currently connected to.
     */
    private final int seeds;
    /**
     * The total rates for all peers for this torrent. These will usually have better
     * precision than summing the rates from all peers. The rates are given as the
     * number of bytes per second.
     */
    private final int downloadSpeed;
    /**
     * The total rates for all peers for this torrent. These will usually have better
     * precision than summing the rates from all peers. The rates are given as the
     * number of bytes per second.
     */
    private final int uploadSpeed;
    /**
     * The number of bytes we have downloaded, only counting the pieces that we actually want
     * to download. i.e. excluding any pieces that we have but have priority 0 (i.e. not wanted).
     */
    private final long downloaded;
    /**
     * The total number of bytes we want to download. This may be smaller than the total
     * torrent size in case any pieces are prioritized to 0, i.e. not wanted.
     */
    private final long totalSize;

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
