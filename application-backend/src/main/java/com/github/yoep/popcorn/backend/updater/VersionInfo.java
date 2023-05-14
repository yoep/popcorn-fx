package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"application", "runtime"})
public class VersionInfo extends Structure implements Closeable {
    public PatchInfo.ByValue application;
    public PatchInfo.ByValue runtime;

    @Override
    public void close() {
        setAutoSynch(false);
        application.close();
        runtime.close();
    }
}
