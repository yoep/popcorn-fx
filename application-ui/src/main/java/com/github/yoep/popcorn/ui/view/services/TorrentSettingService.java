package com.github.yoep.popcorn.ui.view.services;

import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.validation.constraints.NotNull;
import java.text.DecimalFormat;

@Slf4j
@Service
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
    public int toSettingsValue(@NotNull String displayValue) {
        Assert.notNull(displayValue, "displayValue cannot be null");
        // check if the display value is empty
        // if so, return zero
        if (StringUtils.isEmpty(displayValue))
            return 0;

        var kb = Double.parseDouble(displayValue);
        var bytes = Math.round(kb * 1000);

        return Long.valueOf(bytes).intValue();
    }

    //endregion
}
