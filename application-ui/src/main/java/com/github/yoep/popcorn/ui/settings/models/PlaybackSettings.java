package com.github.yoep.popcorn.ui.settings.models;

import lombok.*;
import org.springframework.lang.Nullable;

import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;

import static java.util.Arrays.asList;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class PlaybackSettings extends AbstractSettings {
    public static final String QUALITY_PROPERTY = "quality";
    public static final String FULLSCREEN_PROPERTY = "fullscreen";

    /**
     * The default video playback quality.
     */
    @Nullable
    private Quality quality;
    /**
     * Open the video playback in fullscreen.
     */
    private boolean fullscreen;

    //region Setters

    public void setDefaultQuality(Quality quality) {
        if (Objects.equals(this.quality, quality))
            return;

        var oldValue = this.quality;
        this.quality = quality;
        changes.firePropertyChange(QUALITY_PROPERTY, oldValue, quality);
    }

    public void setFullscreen(boolean fullscreen) {
        if (Objects.equals(this.fullscreen, fullscreen))
            return;

        var oldValue = this.fullscreen;
        this.fullscreen = fullscreen;
        changes.firePropertyChange(FULLSCREEN_PROPERTY, oldValue, fullscreen);
    }

    //endregion

    @Getter
    public enum Quality {
        p480(480),
        p720(720),
        p1080(1080),
        p2160(2160);

        private final int res;

        Quality(int res) {
            this.res = res;
        }

        /**
         * Get the quality for the given value.
         *
         * @param value The value to convert.
         * @return Returns the quality for the given value.
         */
        public static Quality from(String value) {
            var res = Integer.parseInt(value.replaceAll("[a-z]", ""));

            return Arrays.stream(values())
                    .filter(e -> e.getRes() == res)
                    .findFirst()
                    .orElseThrow(() -> new EnumConstantNotPresentException(Quality.class, value));
        }

        /**
         * Get the quality which is below the current one.
         *
         * @return Returns the lower quality if possible, else {@link Optional#empty()} if this is already the lowest quality.
         */
        public Optional<Quality> lower() {
            var qualities = asList(values());
            var index = qualities.indexOf(this) - 1;

            return (index >= 0) ? Optional.of(qualities.get(index)) : Optional.empty();
        }

        /**
         * Get the quality which is above the current one.
         *
         * @return Returns the higher quality if possible, else {@link Optional#empty()} if this is already the highest quality.
         */
        public Optional<Quality> higher() {
            var qualities = asList(values());
            var index = qualities.indexOf(this) + 1;
            var maxIndex = qualities.size() - 1;

            return (index <= maxIndex) ? Optional.of(qualities.get(index)) : Optional.empty();
        }

        @Override
        public String toString() {
            return res + "p";
        }
    }
}
