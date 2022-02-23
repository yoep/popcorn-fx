package com.github.yoep.popcorn.backend.utils;

import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.concurrent.TimeUnit;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class TimeUtils {
    /**
     * Format the given timestamp to a readable time format.
     *
     * @param timestamp The timestamp to format.
     * @return Return the readable time format.
     */
    public static String format(long timestamp) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(timestamp),
                TimeUnit.MILLISECONDS.toSeconds(timestamp) % 60);
    }
}
