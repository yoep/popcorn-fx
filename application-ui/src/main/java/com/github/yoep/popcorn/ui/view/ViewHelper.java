package com.github.yoep.popcorn.ui.view;

import com.github.yoep.popcorn.backend.media.providers.MediaTorrentInfo;
import javafx.scene.control.Tooltip;
import javafx.util.Duration;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Comparator;
import java.util.Map;
import java.util.regex.Pattern;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class ViewHelper {
    private static final Pattern QUALITY_PATTERN = Pattern.compile("(?i)[0-9]+p?");

    /**
     * Create a new instant {@link Tooltip} for the given text.
     * This will create a {@link Tooltip} with {@link Tooltip#setShowDelay(Duration)} of {@link Duration#ZERO},
     * {@link Tooltip#setShowDuration(Duration)} of {@link Duration#INDEFINITE},
     * {@link Tooltip#setHideDelay(Duration)} of {@link Duration#ZERO}.
     *
     * @param text The text of the tooltip.
     * @return Returns the instant Tooltip.
     */
    public static Tooltip instantTooltip(String text) {
        return instantTooltip(new Tooltip(text));
    }

    /**
     * Update the given tooltip so it's shown instantly.
     * This will update the {@link Tooltip} with {@link Tooltip#setShowDelay(Duration)} of {@link Duration#ZERO},
     * {@link Tooltip#setShowDuration(Duration)} of {@link Duration#INDEFINITE},
     * {@link Tooltip#setHideDelay(Duration)} of {@link Duration#ZERO}.
     *
     * @param tooltip The tooltip to update.
     * @return Returns same tooltip instance..
     */
    public static Tooltip instantTooltip(Tooltip tooltip) {
        tooltip.setShowDelay(Duration.ZERO);
        tooltip.setShowDuration(Duration.INDEFINITE);
        tooltip.setHideDelay(Duration.ZERO);
        return tooltip;
    }

    /**
     * @deprecated Use {@link com.github.yoep.popcorn.ui.view.services.VideoQualityService} instead
     */
    @Deprecated
    public static String[] getVideoResolutions(Map<String, MediaTorrentInfo> torrents) {
        return torrents.keySet().stream()
                // filter out the 0 quality
                .filter(e -> !e.equals("0"))
                // filter out any specials, e.g. "3D"
                .filter(e -> QUALITY_PATTERN.matcher(e).matches())
                .sorted(Comparator.comparing(ViewHelper::toResolution))
                .toArray(String[]::new);
    }

    public static Integer toResolution(String quality) {
        return Integer.parseInt(quality.replaceAll("[a-z]", ""));
    }
}
