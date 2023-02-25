package com.github.yoep.popcorn.backend.settings.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.io.Closeable;

@Getter
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"value"})
public class UIScale extends Structure implements Closeable {
    static final String APPENDIX = "%";

    public float value;

    public UIScale() {
    }

    public UIScale(@JsonProperty("value") float value) {
        this.value = value;
    }

    //region Methods

    @Override
    public void close() {
        setAutoSynch(false);
    }

    @Override
    public String toString() {
        return (int) (value * 100) + APPENDIX;
    }

    //endregion
}
