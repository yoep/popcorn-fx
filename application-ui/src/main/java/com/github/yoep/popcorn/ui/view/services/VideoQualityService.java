package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.regex.Pattern;

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

    public CompletableFuture<String> getDefaultVideoResolution(List<String> availableResolutions) {
        return getPlaybackSettings()
                .thenApply(settings -> {
                    if (settings.hasQuality()) {
                        var desiredQuality = settings.getQuality();
                        // check if we can find the request playback quality within the available resolutions
                        var defaultResolution = getResolutionForPlaybackQuality(availableResolutions, desiredQuality)
                                .orElseGet(() -> findLowerQualityResolution(availableResolutions, desiredQuality));

                        // check if the default resolution could be found
                        // if so, return the found resolution
                        // otherwise, return the highest available resolution
                        if (defaultResolution != null)
                            return defaultResolution;
                    }

                    // return the highest resolution by default
                    return availableResolutions.getLast();
                });
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

    private String findLowerQualityResolution(List<String> availableResolutions,
                                              ApplicationSettings.PlaybackSettings.Quality currentQuality) {
        var qualities = List.of(ApplicationSettings.PlaybackSettings.Quality.values());
        int lowerIndex = qualities.indexOf(currentQuality) - 1;

        if (lowerIndex >= 0 && lowerIndex < qualities.size()) {
            return getResolutionForPlaybackQuality(availableResolutions, qualities.get(lowerIndex))
                    .orElse(null);
        }

        return null;
    }
}
