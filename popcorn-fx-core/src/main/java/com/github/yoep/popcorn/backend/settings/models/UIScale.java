package com.github.yoep.popcorn.backend.settings.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.EqualsAndHashCode;
import lombok.Getter;

@Getter
@EqualsAndHashCode
public class UIScale {
    static final String APPENDIX = "%";

    private final float value;

    public UIScale(@JsonProperty("value") float value) {
        this.value = value;
    }

    @Override
    public String toString() {
        return (int) (value * 100) + APPENDIX;
    }
}
