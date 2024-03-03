package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"lastSync"})
public class TrackingSettings extends Structure implements Closeable {
    public static class ByValue extends TrackingSettings implements Structure.ByValue {
    }

    public LastSync.ByReference lastSync;
    
    public Optional<LastSync> getLastSync() {
        return Optional.ofNullable(lastSync);
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(lastSync)
                .ifPresent(LastSync::close);
    }
}
