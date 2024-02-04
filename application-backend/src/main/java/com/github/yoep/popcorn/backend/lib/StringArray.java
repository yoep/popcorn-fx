package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.FxLib;
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

    private List<String> cache;

    public List<String> values() {
        return cache;
    }

    @Override
    public void read() {
        super.read();
        cache = Optional.ofNullable(values)
                .map(e -> e.getStringArray(0, len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLib.INSTANCE.get().dispose_string_array(this);
    }
}
