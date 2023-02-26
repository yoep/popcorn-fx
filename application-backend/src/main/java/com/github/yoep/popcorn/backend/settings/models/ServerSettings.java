package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"apiServer"})
public class ServerSettings extends Structure implements Closeable {
    public static class ByValue extends ServerSettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(ServerSettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.apiServer = settings.apiServer;
        }
    }

    public String apiServer;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
