package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLibInstance;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.io.File;
import java.io.Serializable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleCue} objects which contain the actual parsed data and a reference to the original
 * {@link SubtitleInfo} from which this {@link Subtitle} was generated.
 */
@Getter
@ToString(exclude = {"cached"})
@EqualsAndHashCode(exclude = {"cached"}, callSuper = false)
@Structure.FieldOrder({"filepath", "subtitleInfo", "cueRef", "len"})
public class Subtitle extends Structure implements Serializable, Closeable {
    public String filepath;
    public SubtitleInfo.ByReference subtitleInfo;
    public SubtitleCue.ByReference cueRef;
    public int len;

    private List<SubtitleCue> cached;

    //region Constructors

    public Subtitle() {
    }

    //endregion

    public List<SubtitleCue> getCues() {
        if (cached == null) {
            cacheCues();
        }

        return cached;
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

    public File getFile() {
        return new File(filepath);
    }

    @Override
    public void read() {
        super.read();
        cacheCues();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        FxLibInstance.INSTANCE.get().dispose_subtitle(this);
    }

    private void cacheCues() {
        cached = Optional.ofNullable(cueRef)
                .map(e -> e.toArray(len))
                .map(e -> (SubtitleCue[]) e)
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }
}
