package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.providers.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.view.ViewHelper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Pattern;

@Slf4j
@RequiredArgsConstructor
public class VideoQualityService {
    static final Pattern QUALITY_PATTERN = Pattern.compile("(?i)[0-9]+p?");

    private final ApplicationConfig applicationConfig;

    public String[] getVideoResolutions(Map<String, MediaTorrentInfo> torrents) {
        return torrents.keySet().stream()
                // filter out the 0 quality
                .filter(e -> !e.equals("0"))
                // filter out any specials, e.g. "3D"
                .filter(e -> QUALITY_PATTERN.matcher(e).matches())
                .sorted(Comparator.comparing(ViewHelper::toResolution))
                .toArray(String[]::new);
    }

    public String getDefaultVideoResolution(List<String> availableResolutions) {
        var settings = getPlaybackSettings();
        var defaultQualityOptional = settings.getQuality();

        if (defaultQualityOptional.isPresent()) {
            var defaultQuality = defaultQualityOptional.get();
            // check if we can find the request playback quality within the available resolutions
            var defaultResolution = getResolutionForPlaybackQuality(availableResolutions, defaultQuality)
                    .orElseGet(() -> defaultQuality.lower() // if not found, try the quality below the current one if possible
                            .flatMap(x -> getResolutionForPlaybackQuality(availableResolutions, x))
                            .orElse(null));

            // check if the default resolution could be found
            // if so, return the found resolution
            // otherwise, return the highest available resolution
            if (defaultResolution != null)
                return defaultResolution;
        }

        // return the highest resolution by default
        return availableResolutions.get(availableResolutions.size() - 1);
    }

    private Integer toResolution(String quality) {
        return Integer.parseInt(quality.replaceAll("[a-z]", ""));
    }

    private Optional<String> getResolutionForPlaybackQuality(List<String> availableResolutions, PlaybackSettings.Quality quality) {
        return availableResolutions.stream()
                .filter(e -> toResolution(e) == quality.getRes())
                .findFirst();
    }

    private PlaybackSettings getPlaybackSettings() {
        return applicationConfig.getSettings().getPlaybackSettings();
    }
}
