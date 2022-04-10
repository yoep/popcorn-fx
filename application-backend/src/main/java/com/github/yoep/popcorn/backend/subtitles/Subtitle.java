package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.File;
import java.io.Serializable;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleCue} objects which contain the actual parsed data and a reference to the original
 * {@link SubtitleInfo} from which this {@link Subtitle} was generated.
 */
@Getter
@ToString(exclude = "cues")
@EqualsAndHashCode
public class Subtitle implements Serializable {
    private static final Subtitle NONE = new Subtitle(SubtitleInfo.none());

    private final transient List<SubtitleCue> cues;
    private final SubtitleInfo subtitleInfo;
    private final File file;

    //region Constructors

    Subtitle(SubtitleInfo subtitleInfo) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = null;
        this.cues = Collections.emptyList();
    }

    public Subtitle(File file, List<SubtitleCue> cues) {
        Objects.requireNonNull(file, "file cannot be null");
        Objects.requireNonNull(cues, "cues cannot be null");
        this.subtitleInfo = null;
        this.file = file;
        this.cues = cues;
    }

    public Subtitle(SubtitleInfo subtitleInfo, File file, List<SubtitleCue> cues) {
        Objects.requireNonNull(file, "file cannot be null");
        Objects.requireNonNull(cues, "cues cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = file;
        this.cues = cues;
    }

    //endregion

    /**
     * Get the special subtitle none type.
     *
     * @return Returns the special subtitle none.
     */
    public static Subtitle none() {
        return NONE;
    }

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return getSubtitleInfo()
                .map(SubtitleInfo::isNone)
                .orElse(false);
    }

    /**
     * Check if this subtitle is a custom subtitle.
     *
     * @return Returns true if this subtitle is a custom subtitle, else false.
     */
    public boolean isCustom() {
        return getSubtitleInfo().isEmpty();
    }

    /**
     * Get the subtitle info of this subtitle.
     *
     * @return Returns the subtitle info if present, else {@link Optional#empty()}.
     */
    public Optional<SubtitleInfo> getSubtitleInfo() {
        return Optional.ofNullable(subtitleInfo);
    }

    public Optional<File> getFile() {
        return Optional.ofNullable(file);
    }
}
