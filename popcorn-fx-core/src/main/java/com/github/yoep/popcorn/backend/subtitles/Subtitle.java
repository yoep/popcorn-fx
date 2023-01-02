package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.io.File;
import java.io.Serializable;
import java.util.*;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleCue} objects which contain the actual parsed data and a reference to the original
 * {@link SubtitleInfo} from which this {@link Subtitle} was generated.
 */
@Getter
@ToString(exclude = {"cached"})
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"file", "subtitleInfo", "cueRef", "len", "cap"})
public class Subtitle extends Structure implements Serializable, Closeable {

    private static final Subtitle NONE = new Subtitle(SubtitleInfo.none());

    public String file;
    public SubtitleInfo subtitleInfo;
    public SubtitleCue.ByReference cueRef;
    public int len;
    public int cap;

    private List<SubtitleCue> cached;

    //region Constructors

    public Subtitle() {
    }

    Subtitle(SubtitleInfo subtitleInfo) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = null;
    }

    public Subtitle(File file, List<SubtitleCue> cues) {
        Objects.requireNonNull(file, "file cannot be null");
        Objects.requireNonNull(cues, "cues cannot be null");
        this.subtitleInfo = null;
        this.file = file.getAbsolutePath();
    }

    public Subtitle(SubtitleInfo subtitleInfo, File file, List<SubtitleCue> cues) {
        Objects.requireNonNull(file, "file cannot be null");
        Objects.requireNonNull(cues, "cues cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = file.getAbsolutePath();
    }

    //endregion

    public List<SubtitleCue> getCues() {
        if (cached == null) {
            cached = Optional.ofNullable(cueRef)
                    .map(e -> e.toArray(len))
                    .map(e -> (SubtitleCue[]) e)
                    .map(Arrays::asList)
                    .orElse(Collections.emptyList());
        }

        return cached;
    }

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
     * Get the subtitle info of this subtitle.
     *
     * @return Returns the subtitle info if present, else {@link Optional#empty()}.
     */
    public Optional<SubtitleInfo> getSubtitleInfo() {
        return Optional.ofNullable(subtitleInfo);
    }

    public Optional<File> getFile() {
        return Optional.ofNullable(file)
                .map(File::new);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
