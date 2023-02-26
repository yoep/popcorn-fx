package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;

import java.io.Closeable;
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
    }

    private static final SubtitleInfo NONE = new SubtitleInfo(SubtitleLanguage.NONE);

    public String imdbId;
    public SubtitleLanguage language;
    public SubtitleFile.ByReference files;
    public int len;

    private List<SubtitleFile> cache;

    //region Constructors

    public SubtitleInfo() {
    }

    private SubtitleInfo(SubtitleLanguage language) {
        this.imdbId = null;
        this.language = language;
    }

    @Builder
    public SubtitleInfo(String imdbId, SubtitleLanguage language) {
        this.imdbId = imdbId;
        this.language = language;
    }

    //endregion

    //region Getters & Setters

    /**
     * Get the special "none" subtitle instance.
     * This instance is always the same.
     *
     * @return Returns the special none subtitle.
     */
    public static SubtitleInfo none() {
        return NONE;
    }

    /**
     * Get a new special "custom" subtitle instance.
     * This instance is always unique/new for each invocation.
     *
     * @return Returns the special custom subtitle.
     */
    public static SubtitleInfo custom() {
        return new SubtitleInfo(SubtitleLanguage.CUSTOM);
    }

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
    public Resource getFlagResource() {
        return new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + "/flags/" + language.getCode() + ".png");
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
        cache.forEach(SubtitleFile::close);
    }

    //endregion
}
