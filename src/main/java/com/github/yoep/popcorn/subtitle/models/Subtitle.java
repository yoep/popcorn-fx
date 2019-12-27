package com.github.yoep.popcorn.subtitle.models;

import com.github.spring.boot.javafx.view.ViewLoader;
import lombok.Builder;
import lombok.Data;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;

import java.util.Optional;

@Data
public class Subtitle implements Comparable<Subtitle> {
    public static final String NONE_KEYWORD = "none";

    private final String imdbId;
    private final String language;
    private String url;
    private int score;
    private int downloads;

    public Subtitle(String imdbId, String language, String url) {
        this.imdbId = imdbId;
        this.language = language;
        this.url = url;
    }

    @Builder
    public Subtitle(String imdbId, String language, String url, int score, int downloads) {
        this.imdbId = imdbId;
        this.language = language;
        this.url = url;
        this.score = score;
        this.downloads = downloads;
    }

    /**
     * Get the flag resource for this subtitle.
     *
     * @return Returns the flag resource if found, else {@link Optional#empty()}.
     */
    public Optional<Resource> getFlagResource() {
        ClassPathResource resource = new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + "/flags/" + language + ".png");

        if (resource.exists()) {
            return Optional.of(resource);
        } else {
            return Optional.empty();
        }
    }

    @Override
    public int compareTo(Subtitle compare) {
        if (this.getLanguage().equalsIgnoreCase(NONE_KEYWORD))
            return -1;

        if (compare.getLanguage().equalsIgnoreCase(NONE_KEYWORD))
            return 1;

        return this.getLanguage().compareTo(compare.getLanguage());
    }
}
