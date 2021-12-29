package com.github.yoep.popcorn.backend.subtitles.models;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleException;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

/**
 * The subtitle info contains information about available subtitles for a certain IMDB ID.
 * This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
 */
@Data
@EqualsAndHashCode(of = {"imdbId", "language"})
public class SubtitleInfo implements Comparable<SubtitleInfo>, Serializable {
    private static final SubtitleInfo NONE = new SubtitleInfo(SubtitleLanguage.NONE);

    private final String imdbId;
    private final SubtitleLanguage language;
    private final List<SubtitleFile> files = new ArrayList<>();

    //region Constructors

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

    //region

    /**
     * Add the given subtitle file to the collection of this subtitle info.
     *
     * @param file The file to add.
     */
    public void addFile(SubtitleFile file) {
        Assert.notNull(file, "file cannot be null");
        files.add(file);
    }


    /**
     * Get the subtitle file based on the given matcher.
     * This method search for the best matching file based on the matcher and return it.
     *
     * @param matcher The matched to use for selecting the subtitle file.
     * @return Returns the subtitle file which has the best match.
     */
    public SubtitleFile getFile(SubtitleMatcher matcher) {
        Assert.notNull(matcher, "matcher cannot be null");
        var name = matcher.getName();
        var quality = matcher.getQuality();

        // check if a filename has been given
        // if so, check if we can find an exact match with the subtitle filename
        if (name != null) {
            var matchingFile = findFileByName(name);

            if (matchingFile.isPresent()) {
                return matchingFile.get();
            }
        }

        var matchingFiles = files;

        // check if the quality has been given
        // if so, filter the current list based on the quality
        if (quality != null) {
            matchingFiles = files.stream()
                    .filter(e -> e.getQuality() == null || quality.equals(e.getQuality()))
                    .collect(Collectors.toList());

            // check if anything is found
            // if not, start again from the full list
            if (matchingFiles.size() == 0) {
                matchingFiles = files;
            }
        }

        return findBestScoredFile(matchingFiles);
    }

    //endregion

    //region Functions

    private Optional<SubtitleFile> findFileByName(String name) {
        return files.stream()
                .filter(e -> name.equalsIgnoreCase(e.getName()))
                .sorted() // sort them on best score in case multiple files have been uploaded for the same torrent
                .findFirst();
    }

    private SubtitleFile findBestScoredFile(List<SubtitleFile> subtitleFiles) {
        return subtitleFiles.stream()
                .sorted()
                .findFirst()
                .orElseThrow(() -> new SubtitleException("No best subtitle file could be found for " + this));
    }

    //endregion
}
