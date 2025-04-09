package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.regex.Pattern;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class VideoQualityService {
    static final Pattern QUALITY_PATTERN = Pattern.compile("(?i)[0-9]+p?");

    private final ApplicationConfig applicationConfig;

    public String[] getVideoResolutions(Media.TorrentQuality torrents) {
        return torrents.getQualitiesMap().keySet().stream()
                // filter out the 0 quality
                .filter(e -> !e.equals("0"))
                // filter out any specials, e.g. "3D"
                .filter(e -> QUALITY_PATTERN.matcher(e).matches())
                .sorted(Comparator.comparing(ViewHelper::toResolution))
                .toArray(String[]::new);
    }

    public String getDefaultVideoResolution(List<String> availableResolutions) {
        try {
            return getPlaybackSettings().thenApply(settings -> {
                if (settings.hasQuality()) {
                    var defaultQuality = settings.getQuality();
                    // check if we can find the request playback quality within the available resolutions
                    var defaultResolution = getResolutionForPlaybackQuality(availableResolutions, defaultQuality)
                            .orElseGet(() -> {
                                // if not found, try the quality below the current one if possible
                                var values = asList(ApplicationSettings.PlaybackSettings.Quality.values());
                                var index = values.indexOf(defaultQuality) - 1;

                                if (index < 0 || index >= values.size()) {
                                    return null;
                                }

                                return getResolutionForPlaybackQuality(availableResolutions, values.get(index))
                                        .orElse(null);
                            });

                    // check if the default resolution could be found
                    // if so, return the found resolution
                    // otherwise, return the highest available resolution
                    if (defaultResolution != null)
                        return defaultResolution;
                }

                // return the highest resolution by default
                return availableResolutions.getLast();
            }).get();
        } catch (InterruptedException | ExecutionException e) {
            throw new FxChannelException(e.getMessage(), e);
        }
    }

    private Integer toResolution(String quality) {
        return Integer.parseInt(quality.replaceAll("[a-z]", ""));
    }

    private Optional<String> getResolutionForPlaybackQuality(List<String> availableResolutions, ApplicationSettings.PlaybackSettings.Quality quality) {
        return availableResolutions.stream()
                .filter(e -> toResolution(e) == quality.getNumber())
                .findFirst();
    }

    private CompletableFuture<ApplicationSettings.PlaybackSettings> getPlaybackSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getPlaybackSettings);
    }
}
