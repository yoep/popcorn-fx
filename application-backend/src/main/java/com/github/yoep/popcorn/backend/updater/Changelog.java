package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"features", "featuresLen", "bugfixes", "bugfixesLen"})
public class Changelog extends Structure implements Closeable {
    public Pointer features;
    public int featuresLen;
    public Pointer bugfixes;
    public int bugfixesLen;

    private List<String> featuresCache;
    private List<String> bugfixesCache;

    public List<String> getFeatures() {
        return featuresCache;
    }

    public List<String> getBugfixes() {
        return bugfixesCache;
    }

    @Override
    public void read() {
        super.read();
        featuresCache = Optional.ofNullable(features)
                .map(e -> e.getStringArray(0, featuresLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
        bugfixesCache = Optional.ofNullable(bugfixes)
                .map(e -> e.getStringArray(0, bugfixesLen))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
