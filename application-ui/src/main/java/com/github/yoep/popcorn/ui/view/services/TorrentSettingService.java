package com.github.yoep.popcorn.ui.view.services;

import lombok.extern.slf4j.Slf4j;

import java.text.DecimalFormat;
import java.util.Objects;

@Slf4j
public class TorrentSettingService {

    //region Methods

    /**
     * Convert the given byte value to a human readable display value.
     *
     * @param byteValue The bytes value to convert.
     * @return Returns the display text for the given value.
     */
    public String toDisplayValue(int byteValue) {
        if (byteValue == 0) {
            return "0";
        }

        var format = new DecimalFormat();
        var kb = (float) byteValue / 1000;

        format.setMaximumFractionDigits(2);

        return format.format(kb);
    }

    /**
     * Convert the given display text to a bytes value.
     *
     * @param displayValue The display text to convert.
     * @return Returns the bytes value from the display value.
     */
    public int toSettingsValue(String displayValue) {
        Objects.requireNonNull(displayValue, "displayValue cannot be null");
        // check if the display value is empty
        // if so, return zero
        if (displayValue.isBlank())
            return 0;

        var kb = Double.parseDouble(displayValue);
        var bytes = Math.round(kb * 1000);

        return Long.valueOf(bytes).intValue();
    }

    //endregion
}
