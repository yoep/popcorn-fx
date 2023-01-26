package com.github.yoep.popcorn.backend.media;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"values", "len"})
public class StringArray extends Structure implements Closeable {
    public Pointer values;
    public int len;

    public List<String> values() {
        return Optional.ofNullable(values)
                .map(e -> e.getStringArray(0, len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
