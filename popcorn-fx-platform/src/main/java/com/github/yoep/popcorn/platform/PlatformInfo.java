package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import com.sun.jna.Structure;
import lombok.Getter;

import java.io.Closeable;

@Getter
@Structure.FieldOrder({"type", "arch"})
public class PlatformInfo extends Structure implements com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo, Closeable {
    public PlatformType type;
    public String arch;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
