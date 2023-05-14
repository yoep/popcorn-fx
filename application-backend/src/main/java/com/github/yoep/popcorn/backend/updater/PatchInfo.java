package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"version"})
public class PatchInfo extends Structure implements Closeable {
    public static class ByValue extends PatchInfo implements Structure.ByValue {
    }

    public String version;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
