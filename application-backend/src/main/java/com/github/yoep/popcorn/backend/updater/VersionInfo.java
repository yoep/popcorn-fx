package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"version", "changelog"})
public class VersionInfo extends Structure implements Closeable {
    public String version;
    public Changelog changelog;

    @Override
    public void close() {
        setAutoSynch(false);
        changelog.close();
    }
}