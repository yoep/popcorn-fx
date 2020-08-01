package com.github.yoep.popcorn.ui.settings.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class UIScale {
    private static final String APPENDIX = "%";

    private float value;

    @Override
    public String toString() {
        return (int) (value * 100) + APPENDIX;
    }
}
