package com.github.yoep.popcorn.ui.updater;

import lombok.AllArgsConstructor;
import lombok.Data;

import java.util.Map;

@Data
@AllArgsConstructor
public class VersionInfo {
    private final String version;
    private final Map<String, String> platforms;
}
