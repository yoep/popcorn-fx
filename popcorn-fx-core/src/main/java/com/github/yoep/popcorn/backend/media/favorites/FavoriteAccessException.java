package com.github.yoep.popcorn.backend.media.favorites;

import lombok.Getter;

import java.io.File;
import java.io.IOException;

@Getter
public class FavoriteAccessException extends FavoriteException {
    private final File file;

    public FavoriteAccessException(File file, IOException cause) {
        super("Unable to access favorite file at " + file.getAbsolutePath(), cause);
        this.file = file;
    }
}
