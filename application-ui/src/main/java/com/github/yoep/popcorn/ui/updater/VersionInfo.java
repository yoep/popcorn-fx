package com.github.yoep.popcorn.ui.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

import java.text.MessageFormat;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

@Data
@Builder
@AllArgsConstructor
public class VersionInfo {
    private final String version;
    private final Map<String, String> platforms;

    public Optional<String> downloadForPlatform(PlatformType type, String arch) {
        Objects.requireNonNull(type, "type cannot be empty");
        Objects.requireNonNull(arch, "arch cannot be empty");
        var expectedKey = MessageFormat.format("{0}.{1}", type.getName(), arch);

        return platforms.containsKey(expectedKey) ?
                Optional.of(platforms.get(expectedKey)) : Optional.empty();
    }
}
