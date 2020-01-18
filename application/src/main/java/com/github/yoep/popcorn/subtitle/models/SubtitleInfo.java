package com.github.yoep.popcorn.subtitle.models;

import com.github.spring.boot.javafx.view.ViewLoader;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;

import java.nio.charset.Charset;

@Data
@EqualsAndHashCode(of = {"imdbId", "language"})
public class SubtitleInfo implements Comparable<SubtitleInfo> {
    private static final SubtitleInfo NONE = new SubtitleInfo(SubtitleLanguage.NONE);

    private final String imdbId;
    private final SubtitleLanguage language;
    private String url;
    private int score;
    private int downloads;
    private Charset encoding;

    //region Constructors

    private SubtitleInfo(SubtitleLanguage language) {
        this.imdbId = null;
        this.language = language;
    }

    public SubtitleInfo(String imdbId, SubtitleLanguage language, String url) {
        this.imdbId = imdbId;
        this.language = language;
        this.url = url;
    }

    @Builder
    public SubtitleInfo(String imdbId, SubtitleLanguage language, String url, int score, int downloads, Charset encoding) {
        this.imdbId = imdbId;
        this.language = language;
        this.url = url;
        this.score = score;
        this.downloads = downloads;
        this.encoding = encoding;
    }

    //endregion

    //region Getters

    /**
     * Get the special "none" subtitle instance.
     *
     * @return Returns the special none subtitle.
     */
    public static SubtitleInfo none() {
        return NONE;
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
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link SubtitleLanguage}.
     *
     * @return Returns the flag class path resource.
     */
    public Resource getFlagResource() {
        return new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + "/flags/" + language.getCode() + ".png");
    }

    //endregion

    //region Comparable

    @Override
    public int compareTo(SubtitleInfo compare) {
        if (getLanguage() == SubtitleLanguage.NONE)
            return -1;

        if (compare.getLanguage() == SubtitleLanguage.NONE)
            return 1;

        return this.getLanguage().compareTo(compare.getLanguage());
    }

    //endregion
}
