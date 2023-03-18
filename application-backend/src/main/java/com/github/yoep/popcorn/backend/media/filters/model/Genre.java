package com.github.yoep.popcorn.backend.media.filters.model;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.io.Closeable;
import java.io.Serializable;

@Getter
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"key", "text"})
public class Genre extends Structure implements Comparable<Genre>, Serializable, Closeable {
    public static final String ALL_KEYWORD = "all";
    public static final String MOVIES_KEYWORD = "movies";

    public String key;
    public String text;

    public Genre(String key, String text) {
        this.key = key;
        this.text = text;
    }

    /**
     * Checks with either this genre is the special {@link #ALL_KEYWORD} genre.
     *
     * @return Returns true if this genre is the all genre, else false.
     */
    public boolean isAllGenre() {
        return key.equalsIgnoreCase(ALL_KEYWORD);
    }

    @Override
    public String toString() {
        return text;
    }

    @Override
    public int compareTo(Genre o) {
        // make sure that the "all" key is always on top
        if (this.key.equalsIgnoreCase(ALL_KEYWORD))
            return -1;
        if (o.getKey().equalsIgnoreCase(ALL_KEYWORD))
            return 1;

        return this.getText().compareTo(o.getText());
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
