package com.github.yoep.popcorn.settings.models;

import lombok.*;

import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class PlaybackSettings extends AbstractSettings {
    public static final String QUALITY_PROPERTY = "quality";

    /**
     * The default playback quality.
     */
    @Builder.Default
    private Quality quality = null;

    //region Setters

    public void setDefaultQuality(Quality quality) {
        if (Objects.equals(this.quality, quality))
            return;

        var oldValue = this.quality;
        this.quality = quality;
        changes.firePropertyChange(QUALITY_PROPERTY, oldValue, quality);
    }

    //endregion

    @Getter
    public enum Quality {
        p480(480),
        p720(720),
        p1080(1080);

        private final int res;

        Quality(int res) {
            this.res = res;
        }
    }
}
