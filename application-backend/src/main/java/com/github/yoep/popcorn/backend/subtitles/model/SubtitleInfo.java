package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.io.InputStream;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * The subtitle info contains information about available subtitles for a certain IMDB ID.
 * This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
 */
@Slf4j
@Data
@ToString
@EqualsAndHashCode(of = {"imdbId", "language"}, callSuper = false)
@Structure.FieldOrder({"imdbId", "language", "files", "len"})
public class SubtitleInfo extends Structure implements Closeable {
    public static class ByReference extends SubtitleInfo implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(String imdbId, SubtitleLanguage language) {
            super(imdbId, language);
        }

        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_subtitle_info(this);
        }
    }

    public String imdbId;
    public SubtitleLanguage language;
    public SubtitleFile.ByReference files;
    public int len;

    private List<SubtitleFile> cache;

    //region Constructors

    public SubtitleInfo() {
        super();
    }

    @Builder
    public SubtitleInfo(String imdbId, SubtitleLanguage language) {
        this.imdbId = imdbId;
        this.language = language;
    }

    //endregion

    //region Getters & Setters

    /**
     * Check if this subtitle is a special subtitle.
     *
     * @return Returns true if this subtitle is a special one, else false.
     */
    public boolean isSpecial() {
        return isNone() || isCustom();
    }

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return getLanguage() == SubtitleLanguage.NONE;
    }

    /**
     * Check if this subtitle is the special "custom" subtitle.
     *
     * @return Returns true if this subtitle is the "custom" subtitle, else false.
     */
    public boolean isCustom() {
        return getLanguage() == SubtitleLanguage.CUSTOM;
    }

    /**
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link SubtitleLanguage}.
     *
     * @return Returns the flag class path resource.
     */
    public InputStream getFlagResource() {
        return SubtitleInfo.class.getResourceAsStream( "/images/flags/" + language.getCode() + ".png");
    }

    public List<SubtitleFile> getFiles() {
        return cache;
    }

    //endregion

    //region Methods

    @Override
    public void read() {
        super.read();
        cache = Optional.ofNullable(files)
                .map(e -> (SubtitleFile[]) files.toArray(len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(cache)
                .ifPresent(e -> e.forEach(SubtitleFile::close));
        Optional.ofNullable(files)
                .map(e -> (SubtitleFile.ByReference[]) e.toArray(this.len))
                .stream()
                .flatMap(Arrays::stream)
                .forEach(SubtitleFile::close);
    }

    //endregion
}
