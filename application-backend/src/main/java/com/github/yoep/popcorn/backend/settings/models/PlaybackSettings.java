package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import com.sun.jna.ptr.IntByReference;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;

import static java.util.Arrays.asList;

@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"quality", "fullscreen", "autoPlayNextEpisodeEnabled"})
public class PlaybackSettings extends Structure implements Closeable {
    public static class ByValue extends PlaybackSettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(PlaybackSettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.quality = settings.quality;
            this.fullscreen = settings.fullscreen;
            this.autoPlayNextEpisodeEnabled = settings.autoPlayNextEpisodeEnabled;
        }
    }

    public IntByReference quality;
    public byte fullscreen;
    public byte autoPlayNextEpisodeEnabled;

    public Optional<Quality> getQuality() {
        if (quality != null) {
            return Optional.of(Quality.values()[quality.getValue()]);
        } else {
            return Optional.empty();
        }
    }

    public void setQuality(Quality quality) {
        this.quality = Optional.ofNullable(quality)
                .map(Enum::ordinal)
                .map(IntByReference::new)
                .orElse(null);
    }

    public boolean isFullscreen() {
        return fullscreen == 1;
    }

    public void setFullscreen(boolean fullscreen) {
        this.fullscreen = (byte) (fullscreen ? 1 : 0);
    }

    public boolean isAutoPlayNextEpisodeEnabled() {
        return autoPlayNextEpisodeEnabled == 1;
    }

    public void setAutoPlayNextEpisodeEnabled(boolean autoPlayNextEpisodeEnabled) {
        this.autoPlayNextEpisodeEnabled = (byte) (autoPlayNextEpisodeEnabled ? 1 : 0);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

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

        @Override
        public String toString() {
            return res + "p";
        }
    }
}
