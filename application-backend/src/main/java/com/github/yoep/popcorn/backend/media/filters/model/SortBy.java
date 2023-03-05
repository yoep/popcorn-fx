package com.github.yoep.popcorn.backend.media.filters.model;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.io.Closeable;

@Getter
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"key", "text"})
public class SortBy extends Structure implements Closeable {
    public String key;
    public String text;

    public SortBy(String key, String text) {
        this.key = key;
        this.text = text;
    }

    @Override
    public String toString() {
        return text;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
